// SPDX-License-Identifier: MPL-2.0

mod app;
mod config;
mod i18n;
mod install;
mod overlay;
mod pipewire;
mod voxtype;

fn main() -> cosmic::iced::Result {
    if let Some(command) = std::env::args()
        .skip(1)
        .find(|arg| matches!(arg.as_str(), "--install" | "--uninstall"))
    {
        let result = match command.as_str() {
            "--install" => install::install(),
            _ => install::uninstall(),
        };
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }

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
