//! Parse falcond status file (`/tmp/falcond_status`).
//!
//! The status file uses a simple `KEY: VALUE` / `  KEY: VALUE` format
//! with section headers like `FEATURES:`, `CONFIG:`, `CURRENT_STATUS:`.

use std::collections::HashMap;
use std::path::Path;

/// Falcond status file path (tmp, world-readable).
pub const STATUS_PATH: &str = "/tmp/falcond_status";

/// Parsed falcond daemon status.
#[derive(Debug, Clone, Default)]
pub struct FalcondStatus {
    /// Whether performance mode switching is available.
    pub performance_available: bool,
    /// Profile mode (none / handheld / htpc).
    pub profile_mode: String,
    /// Global `VCache` mode from config.
    pub config_vcache: String,
    /// Global SCX scheduler from config.
    pub config_scx: String,
    /// Number of loaded profiles.
    pub loaded_profiles: u32,
    /// Currently active profile name (if any).
    pub active_profile: Option<String>,
    /// Live: performance mode active/inactive.
    pub perf_mode_active: bool,
    /// Live: current `VCache` mode.
    pub current_vcache: String,
    /// Live: current SCX scheduler.
    pub current_scx: String,
    /// Live: screensaver inhibit active.
    pub screensaver_inhibited: bool,
}

/// Parse the falcond status file.
///
/// Returns `None` if the file doesn't exist or is unreadable.
#[must_use]
pub fn read() -> Option<FalcondStatus> {
    let content = std::fs::read_to_string(Path::new(STATUS_PATH)).ok()?;
    Some(parse(&content))
}

/// Parse status file content into structured data.
#[must_use]
pub fn parse(content: &str) -> FalcondStatus {
    let mut status = FalcondStatus::default();
    let mut section = "";

    // Pre-parse key-value pairs per section
    let mut kv: HashMap<(&str, &str), &str> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Section header: "SECTION_NAME:"
        if !trimmed.starts_with(' ') && trimmed.ends_with(':') && !trimmed.contains(": ") {
            section = trimmed.trim_end_matches(':');
            continue;
        }

        // Top-level key: "KEY: VALUE"
        if !line.starts_with(' ') {
            if let Some((key, val)) = trimmed.split_once(": ") {
                kv.insert(("", key), val);
            }
            continue;
        }

        // Indented key-value: "  Key: Value"
        if let Some((key, val)) = trimmed.split_once(": ") {
            kv.insert((section, key), val);
        }
    }

    // Map parsed values to struct fields
    if let Some(&v) = kv.get(&("FEATURES", "Performance Mode")) {
        status.performance_available = v == "Available";
    }
    if let Some(&v) = kv.get(&("CONFIG", "Profile Mode")) {
        v.clone_into(&mut status.profile_mode);
    }
    if let Some(&v) = kv.get(&("CONFIG", "Global VCache Mode")) {
        v.clone_into(&mut status.config_vcache);
    }
    if let Some(&v) = kv.get(&("CONFIG", "Global SCX Scheduler")) {
        v.clone_into(&mut status.config_scx);
    }
    if let Some(&v) = kv.get(&("", "LOADED_PROFILES")) {
        status.loaded_profiles = v.parse().unwrap_or(0);
    }
    if let Some(&v) = kv.get(&("", "ACTIVE_PROFILE")) {
        status.active_profile = if v == "None" {
            None
        } else {
            Some(v.to_owned())
        };
    }
    if let Some(&v) = kv.get(&("CURRENT_STATUS", "Performance Mode")) {
        status.perf_mode_active = v == "Active";
    }
    if let Some(&v) = kv.get(&("CURRENT_STATUS", "VCache Mode")) {
        v.clone_into(&mut status.current_vcache);
    }
    if let Some(&v) = kv.get(&("CURRENT_STATUS", "SCX Scheduler")) {
        v.clone_into(&mut status.current_scx);
    }
    if let Some(&v) = kv.get(&("CURRENT_STATUS", "Screensaver Inhibit")) {
        status.screensaver_inhibited = v == "Active";
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_status() {
        let input = "\
FEATURES:
  Performance Mode: Available

CONFIG:
  Profile Mode: none
  Global VCache Mode: cache
  Global SCX Scheduler: bpfland

AVAILABLE_SCX_SCHEDULERS:
  - scx_bpfland
  - scx_lavd

LOADED_PROFILES: 5

ACTIVE_PROFILE: Cyberpunk2077.exe

QUEUED_PROFILES:
  (None)

RESTORE_STATE:
  SCX Scheduler: none (Mode: default)
  Power Profile: balanced

CURRENT_STATUS:
  Performance Mode: Active
  VCache Mode: cache
  SCX Scheduler: bpfland
  Screensaver Inhibit: Active
";
        let s = parse(input);
        assert!(s.performance_available);
        assert_eq!(s.profile_mode, "none");
        assert_eq!(s.config_vcache, "cache");
        assert_eq!(s.config_scx, "bpfland");
        assert_eq!(s.loaded_profiles, 5);
        assert_eq!(s.active_profile.as_deref(), Some("Cyberpunk2077.exe"));
        assert!(s.perf_mode_active);
        assert_eq!(s.current_vcache, "cache");
        assert_eq!(s.current_scx, "bpfland");
        assert!(s.screensaver_inhibited);
    }

    #[test]
    fn parse_inactive_status() {
        let input = "\
FEATURES:
  Performance Mode: Unavailable

CONFIG:
  Profile Mode: handheld
  Global VCache Mode: none
  Global SCX Scheduler: none

LOADED_PROFILES: 0

ACTIVE_PROFILE: None
";
        let s = parse(input);
        assert!(!s.performance_available);
        assert_eq!(s.profile_mode, "handheld");
        assert_eq!(s.active_profile, None);
        assert_eq!(s.loaded_profiles, 0);
        assert!(!s.perf_mode_active);
    }
}
