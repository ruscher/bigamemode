//! "Booster Ignition" toggle: animated switch controlling global performance mode.
//!
//! When activated, switches `PowerProfiles` to "performance" via D-Bus.
//! When deactivated, restores "balanced".

use libadwaita as adw;
use adw::prelude::*;
use gtk4::{gio, glib};

use crate::i18n::i18n;

/// Build the Booster Mode toggle row with CSS glow animation.
///
/// Connects to `PowerProfiles` D-Bus daemon on toggle.
#[must_use]
pub fn build() -> adw::SwitchRow {
    let row = adw::SwitchRow::builder()
        .title(i18n("Booster Mode"))
        .subtitle(i18n("Activate system-wide gaming optimizations"))
        .build();

    row.add_css_class("booster-idle");

    // Set initial state from current power profile
    let init_row = row.clone();
    glib::spawn_future_local(async move {
        let current = gio::spawn_blocking(bigame_core::dbus::power_profile_get)
            .await
            .ok()
            .flatten();
        if current.as_deref() == Some("performance") {
            init_row.set_active(true);
        }
    });

    row.connect_active_notify(|row| {
        let profile = if row.is_active() {
            row.remove_css_class("booster-idle");
            row.add_css_class("booster-active");
            "performance"
        } else {
            row.remove_css_class("booster-active");
            row.add_css_class("booster-idle");
            "balanced"
        };

        // Switch PowerProfiles via D-Bus (background thread)
        let target = profile.to_owned();
        gio::spawn_blocking(move || {
            let _ = bigame_core::dbus::power_profile_set(&target);
        });
    });

    row
}
