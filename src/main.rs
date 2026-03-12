use {
    gpui::{
        AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size,
    },
    tracing::info,
    tracing_subscriber::{EnvFilter, fmt},
};

mod error;
mod fs;
mod ops;
mod ui;

static PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
static FONT: &[u8] = include_bytes!("../assets/0xProtoNerdFont-Regular.ttf");

fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!("Starting {}", PACKAGE_NAME);

    Application::new().run(|cx: &mut gpui::App| {
        let font = FONT.iter().cloned().collect();
        cx.text_system().add_fonts(vec![font]).ok();

        let bounds = Bounds::centered(None, size(px(900.0), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some(PACKAGE_NAME.into()),
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(bounds)),

                ..Default::default()
            },
            |_, cx| cx.new(ui::Explorer::new),
        )
        .unwrap();
    });

    Ok(())
}
