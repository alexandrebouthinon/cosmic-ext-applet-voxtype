// SPDX-License-Identifier: MPL-2.0

//! Provides localization support for this crate.

use std::sync::LazyLock;

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
    unic_langid::LanguageIdentifier,
};
use rust_embed::RustEmbed;

/// Applies the requested language(s) to requested translations from the `fl!()` macro.
pub fn init(requested_languages: &[LanguageIdentifier]) {
    if let Err(why) = localizer().select(requested_languages) {
        eprintln!("error while loading fluent localizations: {why}");
    }
}

// Get the `Localizer` to be used for localizing this library.
#[must_use]
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

/// Request a localized string by ID from the i18n/ directory.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::i18n::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// Collects the message ids defined for `language` in `i18n/<language>/`.
    ///
    /// Uses a loader local to the test (rather than the shared `LANGUAGE_LOADER`)
    /// so parallel tests never race on global state.
    fn message_ids(language: &str) -> HashSet<String> {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        let language_id: LanguageIdentifier = language.parse().expect("valid language tag");

        loader
            .load_languages(&Localizations, &[language_id.clone()])
            .expect("failed to load language");

        loader.with_message_iter(&language_id, |iter| {
            iter.map(|message| message.id.name.to_owned()).collect()
        })
    }

    #[test]
    fn fr_and_en_have_the_same_message_ids() {
        assert_eq!(
            message_ids("en"),
            message_ids("fr"),
            "fr and en fluent files must expose the exact same message ids"
        );
    }

    #[test]
    fn french_locales_select_the_french_translation() {
        for tag in ["fr-CA", "fr-FR"] {
            let loader: FluentLanguageLoader = fluent_language_loader!();
            let requested: Vec<LanguageIdentifier> = vec![tag.parse().expect("valid language tag")];

            i18n_embed::select(&loader, &Localizations, &requested).expect("selection failed");

            assert_eq!(
                loader.get("tab-status"),
                "État",
                "locale {tag} should select the French translation"
            );
        }
    }

    #[test]
    fn other_locales_fall_back_to_english() {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        let requested: Vec<LanguageIdentifier> = vec!["de-DE".parse().expect("valid language tag")];
        i18n_embed::select(&loader, &Localizations, &requested).expect("selection failed");
        assert_eq!(loader.get("tab-status"), "Status");

        // The POSIX "C" locale is not a valid BCP-47 tag: `DesktopLanguageRequester`
        // filters it out of `requested_languages()`, so no language is requested.
        let loader: FluentLanguageLoader = fluent_language_loader!();
        i18n_embed::select(&loader, &Localizations, &[]).expect("selection failed");
        assert_eq!(loader.get("tab-status"), "Status");
    }
}
