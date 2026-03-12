use {
    crate::{
        fs,
        ui::{
            Explorer,
            constants::*,
            types::{Modal, ZipEntry},
            utils::{file_icon, file_icon_color},
        },
    },
    gpui::{
        ClickEvent, Context, ElementId, InteractiveElement, IntoElement, ParentElement,
        StatefulInteractiveElement, Styled, div, px, rgb,
    },
};

pub fn render_zip_filelist(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let zv = match &this.zip_view {
        Some(zv) => zv.clone(),
        None => return div().flex_1().into_any_element(),
    };

    let header = div()
        .flex()
        .items_center()
        .px_3()
        .py_1()
        .bg(rgb(BG_TOOLBAR))
        .border_b_1()
        .border_color(rgb(COL_BORDER))
        .text_xs()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(rgb(COL_MUTED))
        .child(div().w(px(44.0)))
        .child(div().flex_1().child("Name"))
        .child(div().w(px(90.0)).child("Size"))
        .child(div().w(px(90.0)).child("Compressed"));

    let zip_path_btn = zv.zip_path.clone();
    let toolbar = div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py(px(4.0))
        .bg(rgb(BG_SIDEBAR))
        .border_b_1()
        .border_color(rgb(COL_BORDER))
        .text_xs()
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(COL_ZIP))
                .child(format!("{ICO_ARCHIVE} Browsing archive")),
        )
        .child(div().flex_1())
        .child(
            div()
                .id("extract-all-btn")
                .px_3()
                .py(px(3.0))
                .rounded_md()
                .cursor_pointer()
                .bg(rgb(BG_MENU))
                .text_color(rgb(COL_ZIP))
                .border_1()
                .border_color(rgb(COL_ZIP))
                .hover(|d| d.bg(rgb(BG_MENU_HOVER)))
                .text_size(px(11.0))
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    let dest = zip_path_btn
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| this.current_dir.clone());
                    this.modal = Some(Modal::ExtractZip {
                        zip_path: zip_path_btn.clone(),
                        dest,
                    });
                    cx.notify();
                }))
                .child(format!("{ICO_EXTRACT} Extract All")),
        );

    let mut rows = div().id("zip-list-scroll").flex_1().overflow_y_scroll();

    let entries = zv.entries.clone();
    for (i, ze) in entries.iter().enumerate() {
        let icon = if ze.is_dir {
            ICO_FOLDER
        } else {
            file_icon(&ze.name)
        };
        let icon_color = if ze.is_dir {
            COL_DIR
        } else {
            file_icon_color(&ze.name)
        };
        let display_name: gpui::SharedString = ze.name.clone().into();
        let size_str: gpui::SharedString = if ze.is_dir {
            "—".into()
        } else {
            fs::fmt_size(ze.size).into()
        };
        let comp_str: gpui::SharedString = if ze.is_dir {
            "—".into()
        } else {
            fs::fmt_size(ze.compressed_size).into()
        };
        let is_dir = ze.is_dir;
        let subdir = ze.name.clone() + "/";

        rows = rows.child(
            div()
                .id(ElementId::Name(format!("zrow-{i}").into()))
                .flex()
                .items_center()
                .px_3()
                .py(px(3.0))
                .cursor_pointer()
                .hover(|d| d.bg(rgb(BG_ROW_HOVER)))
                .border_b_1()
                .border_color(rgb(0x1E1E2E))
                .on_click(cx.listener(move |this, ev: &ClickEvent, _, cx| {
                    if ev.click_count() >= 2 && is_dir {
                        this.zip_navigate_into_dir(&subdir, cx);
                    }
                }))
                .child(div().w(px(24.0)))
                .child(div().w(px(20.0)).text_color(rgb(icon_color)).child(icon))
                .child(
                    div()
                        .flex_1()
                        .truncate()
                        .text_color(rgb(if is_dir { COL_DIR } else { COL_TEXT }))
                        .child(display_name),
                )
                .child(div().w(px(90.0)).text_color(rgb(COL_MUTED)).child(size_str))
                .child(div().w(px(90.0)).text_color(rgb(COL_MUTED)).child(comp_str)),
        );
    }

    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(toolbar)
        .child(header)
        .child(rows)
        .into_any_element()
}

pub fn zip_entries_for_dir(
    zip_path: &std::path::PathBuf,
    inner_dir: &str,
) -> std::io::Result<Vec<ZipEntry>> {
    use std::collections::BTreeMap;

    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let mut dirs: BTreeMap<String, bool> = BTreeMap::new();
    let mut files: Vec<ZipEntry> = vec![];

    for i in 0..archive.len() {
        let Ok(f) = archive.by_index_raw(i) else {
            continue;
        };
        let name = f.name();

        if !name.starts_with(inner_dir) {
            continue;
        }
        let rest = &name[inner_dir.len()..];
        if rest.is_empty() {
            continue;
        }

        if let Some(slash) = rest.find('/') {
            let dir_name = rest[..slash].to_string();
            dirs.entry(dir_name).or_insert(true);
        } else {
            files.push(ZipEntry {
                name: rest.to_string(),
                is_dir: false,
                size: f.size(),
                compressed_size: f.compressed_size(),
            });
        }
    }

    let mut result: Vec<ZipEntry> = dirs
        .into_keys()
        .map(|name| ZipEntry {
            name,
            is_dir: true,
            size: 0,
            compressed_size: 0,
        })
        .collect();
    result.extend(files);
    Ok(result)
}

pub fn extract_zip(
    zip_path: &std::path::PathBuf,
    dest: &std::path::PathBuf,
) -> std::io::Result<()> {
    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    archive
        .extract(dest)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

pub fn parent_zip_dir(inner: &str) -> Option<String> {
    let trimmed = inner.trim_end_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    if let Some(pos) = trimmed.rfind('/') {
        Some(format!("{}/", &trimmed[..pos]))
    } else {
        Some(String::new())
    }
}
