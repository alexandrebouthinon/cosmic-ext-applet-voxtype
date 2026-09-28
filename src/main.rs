// SPDX-License-Identifier: MPL-2.0

mod app;
mod config;
mod i18n;
mod overlay;
mod voxtype;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    if std::env::args().any(|a| a == "--overlay") {
        let settings = cosmic::app::Settings::default().no_main_window(true);
        return cosmic::app::run::<overlay::Overlay>(settings, ());
    }

    // Starts the applet's event loop with `()` as the application's flags.
    cosmic::applet::run::<app::AppModel>(())
}
