//! sched-ext BPF scheduler management.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Supported sched-ext schedulers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scheduler {
    /// No sched-ext scheduler (kernel default).
    #[default]
    None,
    /// BPF land scheduler — general-purpose, low-latency.
    Bpfland,
    /// LAVD scheduler — latency-aware virtual deadline.
    Lavd,
    /// Rusty scheduler — Rust-based with load balancing.
    Rusty,
    /// Flash scheduler — ultra-low latency.
    Flash,
}

impl fmt::Display for Scheduler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => f.write_str("none"),
            Self::Bpfland => f.write_str("bpfland"),
            Self::Lavd => f.write_str("lavd"),
            Self::Rusty => f.write_str("rusty"),
            Self::Flash => f.write_str("flash"),
        }
    }
}

/// Scheduler operating mode / tuning preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchedMode {
    /// Balanced default.
    #[default]
    Default,
    /// Optimized for gaming workloads.
    Gaming,
    /// Power-saving mode.
    Power,
    /// Minimum-latency mode.
    Latency,
    /// Server/throughput mode.
    Server,
}

impl fmt::Display for SchedMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => f.write_str("default"),
            Self::Gaming => f.write_str("gaming"),
            Self::Power => f.write_str("power"),
            Self::Latency => f.write_str("latency"),
            Self::Server => f.write_str("server"),
        }
    }
}

/// Detect installed sched-ext schedulers by scanning `/usr/bin/scx_*`.
///
/// Returns a sorted list of scheduler names (e.g. `["bpfland", "lavd", "rusty"]`).
/// Always includes "none" as the first entry.
#[must_use]
pub fn detect_installed() -> Vec<String> {
    let mut schedulers = vec!["none".to_owned()];
    if let Ok(entries) = std::fs::read_dir("/usr/bin") {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(sched) = name.strip_prefix("scx_") {
                if !sched.is_empty() {
                    schedulers.push(sched.to_owned());
                }
            }
        }
    }
    schedulers.sort();
    schedulers.dedup();
    schedulers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_display() {
        assert_eq!(Scheduler::None.to_string(), "none");
        assert_eq!(Scheduler::Bpfland.to_string(), "bpfland");
        assert_eq!(Scheduler::Lavd.to_string(), "lavd");
        assert_eq!(Scheduler::Rusty.to_string(), "rusty");
        assert_eq!(Scheduler::Flash.to_string(), "flash");
    }

    #[test]
    fn sched_mode_display() {
        assert_eq!(SchedMode::Default.to_string(), "default");
        assert_eq!(SchedMode::Gaming.to_string(), "gaming");
        assert_eq!(SchedMode::Power.to_string(), "power");
        assert_eq!(SchedMode::Latency.to_string(), "latency");
        assert_eq!(SchedMode::Server.to_string(), "server");
    }

    #[test]
    fn scheduler_serde_round_trip() {
        let json = serde_json::to_string(&Scheduler::Bpfland).unwrap();
        assert_eq!(json, "\"bpfland\"");
        let parsed: Scheduler = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, Scheduler::Bpfland);
    }

    #[test]
    fn sched_mode_serde_round_trip() {
        let json = serde_json::to_string(&SchedMode::Gaming).unwrap();
        assert_eq!(json, "\"gaming\"");
        let parsed: SchedMode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, SchedMode::Gaming);
    }

    #[test]
    fn default_scheduler() {
        assert_eq!(Scheduler::default(), Scheduler::None);
    }

    #[test]
    fn default_sched_mode() {
        assert_eq!(SchedMode::default(), SchedMode::Default);
    }
}
