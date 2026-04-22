//! AMD 3D V-Cache mode control via sysfs.
//!
//! Sysfs path: `/sys/bus/platform/drivers/amd_x3d_vcache/AMDI0101:00/amd_x3d_mode`

use std::path::Path;

use anyhow::{Context, Result};

/// Sysfs path for AMD X3D `VCache` mode.
const SYSFS_PATH: &str =
    "/sys/bus/platform/drivers/amd_x3d_vcache/AMDI0101:00/amd_x3d_mode";

/// Check whether `VCache` hardware is present.
#[must_use]
pub fn is_available() -> bool {
    Path::new(SYSFS_PATH).exists()
}

/// Read current `VCache` mode from sysfs.
///
/// Returns "frequency", "cache", or the raw sysfs string.
/// Returns `None` if hardware is unavailable.
#[must_use]
pub fn read_mode() -> Option<String> {
    std::fs::read_to_string(SYSFS_PATH)
        .ok()
        .map(|s| s.trim().to_owned())
}

/// Write `VCache` mode to sysfs via sudo.
///
/// # Errors
/// Returns error if sysfs write or sudo fails.
///
/// # Panics
/// Does not panic — all command failures are returned as errors.
pub fn set_mode(mode: &str) -> Result<()> {
    anyhow::ensure!(
        matches!(mode, "frequency" | "cache"),
        "invalid VCache mode: {mode}"
    );

    let status = std::process::Command::new("sudo")
        .args(["-n", "/usr/bin/tee", SYSFS_PATH])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .and_then(|mut child| {
            if let Some(ref mut stdin) = child.stdin {
                use std::io::Write;
                stdin.write_all(mode.as_bytes())?;
            }
            child.wait()
        })
        .context("sudo tee vcache sysfs")?;

    anyhow::ensure!(status.success(), "sudo tee vcache failed: {status}");
    Ok(())
}
