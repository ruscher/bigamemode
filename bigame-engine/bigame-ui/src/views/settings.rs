//! Settings view: user preferences for appearance, notifications, and telemetry.

use adw::prelude::*;
use libadwaita as adw;

use crate::i18n::i18n;
use crate::settings;

/// Build the Settings preferences page.
#[must_use]
pub fn build() -> adw::PreferencesPage {
    let page = adw::PreferencesPage::new();
    let current = settings::load();

    // Appearance group
    let appearance = adw::PreferencesGroup::new();
    appearance.set_title(&i18n("Appearance"));

    let dark_row = adw::SwitchRow::builder()
        .title(i18n("Dark Mode"))
        .subtitle(i18n("Force dark color scheme"))
        .active(current.dark_mode)
        .build();
    dark_row.connect_active_notify(|row| {
        let mgr = adw::StyleManager::default();
        let scheme = if row.is_active() {
            adw::ColorScheme::ForceDark
        } else {
            adw::ColorScheme::Default
        };
        mgr.set_color_scheme(scheme);

        let mut s = settings::load();
        s.dark_mode = row.is_active();
        settings::save(&s);
    });
    appearance.add(&dark_row);
    page.add(&appearance);

    // Apply stored dark mode on build
    if current.dark_mode {
        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    }

    // Notifications group
    let notif = adw::PreferencesGroup::new();
    notif.set_title(&i18n("Notifications"));

    let notif_row = adw::SwitchRow::builder()
        .title(i18n("Game Notifications"))
        .subtitle(i18n("Show notifications on game launch and exit"))
        .active(current.notifications_enabled)
        .build();
    notif_row.connect_active_notify(|row| {
        let mut s = settings::load();
        s.notifications_enabled = row.is_active();
        settings::save(&s);
    });
    notif.add(&notif_row);
    page.add(&notif);

    // Telemetry group
    let telemetry = adw::PreferencesGroup::new();
    telemetry.set_title(&i18n("Telemetry"));

    let ping_row = adw::EntryRow::builder()
        .title(i18n("Ping Target"))
        .text(&current.ping_target)
        .build();
    ping_row.connect_changed(|row| {
        let text = row.text().to_string();
        if !text.is_empty() {
            let mut s = settings::load();
            s.ping_target = text;
            settings::save(&s);
        }
    });
    telemetry.add(&ping_row);
    page.add(&telemetry);

    // About group — app identity with icon + version
    let about = adw::PreferencesGroup::new();
    about.set_title(&i18n("About"));

    let app_icon =
        gtk4::Image::from_resource("/com/biglinux/BiGameMode/icons/com.biglinux.BiGameMode.svg");
    app_icon.set_pixel_size(32);
    let version_row = adw::ActionRow::builder()
        .title("BiGame-mode")
        .subtitle(format!("v{}", env!("CARGO_PKG_VERSION")))
        .build();
    version_row.add_prefix(&app_icon);

    // Button to open the full About dialog
    let about_btn = gtk4::Button::builder()
        .icon_name("help-about-symbolic")
        .tooltip_text(i18n("About BiGame-mode"))
        .valign(gtk4::Align::Center)
        .css_classes(["flat"])
        .build();
    about_btn.connect_clicked(|_| {
        if let Some(app) = gtk4::gio::Application::default() {
            app.activate_action("about", None);
        }
    });
    version_row.add_suffix(&about_btn);

    about.add(&version_row);
    page.add(&about);

    page
}
