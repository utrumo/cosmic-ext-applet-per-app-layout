mod app;
mod i18n;
mod wayland_handler;
mod wayland_state;
mod wayland_subscription;

fn main() -> cosmic::iced::Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
    i18n::init(&requested_languages);

    app::run()
}
