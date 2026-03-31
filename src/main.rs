mod app;
mod i18n;
mod wayland_handler;
mod wayland_state;
mod wayland_subscription;
mod xkb;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    if std::env::args().any(|a| a == "--register") {
        app::register();
        return Ok(());
    }

    if std::env::args().any(|a| a == "--unregister") {
        app::unregister();
        return Ok(());
    }

    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    app::run()
}
