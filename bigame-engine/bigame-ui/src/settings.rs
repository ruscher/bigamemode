//! User settings persistence (window geometry, last tab).
//!
//! Settings stored as TOML in `$XDG_CONFIG_HOME/bigame-mode/settings.toml`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Persisted user preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub window_width: i32,
    pub window_height: i32,
    pub maximized: bool,
    pub last_tab: String,
    /// Force dark mode (true = dark, false = system default).
    pub dark_mode: bool,
    /// Enable desktop notifications on game launch/exit.
    pub notifications_enabled: bool,
    /// Ping target for network latency telemetry.
    pub ping_target: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            window_width: 800,
            window_height: 600,
            maximized: false,
            last_tab: String::from("dashboard"),
            dark_mode: false,
            notifications_enabled: true,
            ping_target: String::from("1.1.1.1"),
        }
    }
}

/// Settings file path: `$XDG_CONFIG_HOME/bigame-mode/settings.toml`.
fn settings_path() -> PathBuf {
    let config = std::env::var("XDG_CONFIG_HOME").map_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/tmp"));
            PathBuf::from(home).join(".config")
        }, PathBuf::from);
    config.join("bigame-mode").join("settings.toml")
}

/// Load settings from disk. Returns defaults on any error.
#[must_use]
pub fn load() -> Settings {
    let path = settings_path();
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

/// Save settings to disk. Silently ignores errors.
pub fn save(settings: &Settings) {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(settings) {
        let _ = std::fs::write(&path, content);
    }
}
