//! Polkit authorization helpers for privileged operations.

/// Polkit action identifiers for BiGame-mode.
pub mod actions {
    /// Set CPU frequency governor.
    pub const SET_GOVERNOR: &str = "com.biglinux.bigamemode.set-governor";
    /// Change sched-ext scheduler.
    pub const SET_SCHEDULER: &str = "com.biglinux.bigamemode.set-scheduler";
    /// Set `VCache` mode.
    pub const SET_VCACHE: &str = "com.biglinux.bigamemode.set-vcache";
    /// Write falcond daemon config.
    pub const WRITE_CONFIG: &str = "com.biglinux.bigamemode.write-config";
    /// Save or delete game profiles.
    pub const MANAGE_PROFILES: &str = "com.biglinux.bigamemode.manage-profiles";
}
