//! Internationalization (gettext) setup for BiGame-mode UI.

use gettextrs::{LocaleCategory, gettext};

/// Application gettext domain.
const GETTEXT_DOMAIN: &str = "bigame-mode";

/// Default locale directory for installed .mo files.
const LOCALE_DIR: &str = "/usr/share/locale";

/// Initialize gettext for the application.
///
/// Respects `BIGAME_LOCALEDIR` env var for development/testing.
/// Falls back to `/usr/share/locale` when not set (production).
///
/// # Panics
/// Panics if locale binding fails (missing system locale support).
pub fn init() {
    let locale_dir = std::env::var("BIGAME_LOCALEDIR").unwrap_or_else(|_| LOCALE_DIR.to_owned());
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_DOMAIN, &locale_dir).expect("bindtextdomain");
    gettextrs::textdomain(GETTEXT_DOMAIN).expect("textdomain");
}

/// Translate a string via gettext.
#[must_use]
pub fn i18n(s: &str) -> String {
    gettext(s)
}
