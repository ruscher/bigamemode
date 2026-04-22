//! CPU frequency governor management via sysfs.

use anyhow::{Context, Result};
use tokio::fs;

/// Set the CPU frequency governor for all online cores via pkexec tee.
///
/// Enumerates `/sys/devices/system/cpu/cpuN/cpufreq/scaling_governor` for all
/// cores present in sysfs, then writes `governor` to all of them in a single
/// `pkexec tee` invocation (reduces auth prompts with `auth_admin_keep`).
///
/// # Errors
/// Returns error if no governor paths are found, the governor name is invalid,
/// or pkexec fails.
pub fn set(governor: &str) -> Result<()> {
    // Strict validation: [a-zA-Z0-9_] only — prevents injection.
    anyhow::ensure!(
        !governor.is_empty() && governor.chars().all(|c| c.is_alphanumeric() || c == '_'),
        "invalid governor name: {governor}"
    );

    let paths = governor_paths();
    anyhow::ensure!(!paths.is_empty(), "no CPUs with governor control found in sysfs");

    // Single pkexec tee writing to all paths at once.
    // Path strings come from sysfs enumeration, not user input → no injection.
    let mut cmd = std::process::Command::new("sudo"); cmd.arg("-n");
    cmd.arg("tee");
    for p in &paths {
        cmd.arg(p);
    }
    cmd.stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null());

    let mut child = cmd.spawn().context("spawn pkexec tee governor")?;
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(governor.as_bytes()).context("write governor to pkexec stdin")?;
    }

    let exit = child.wait().context("wait pkexec tee governor")?;
    anyhow::ensure!(exit.success(), "pkexec tee governor failed: {exit}");
    Ok(())
}

/// Collect all `scaling_governor` sysfs paths for online CPUs.
fn governor_paths() -> Vec<String> {
    let base = "/sys/devices/system/cpu";
    let Ok(dir) = std::fs::read_dir(base) else {
        return Vec::new();
    };
    let mut paths: Vec<String> = dir
        .flatten()
        .filter_map(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            // Match cpuN entries (cpu0, cpu1, …) — exclude cpufreq, cpuidle, etc.
            if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                let path = format!("{base}/{name}/cpufreq/scaling_governor");
                if std::path::Path::new(&path).exists() {
                    return Some(path);
                }
            }
            None
        })
        .collect();
    paths.sort();
    paths
}

/// List available CPU governors from sysfs.
///
/// # Errors
/// Returns error if sysfs path is unreadable.
pub async fn available() -> Result<Vec<String>> {
    let content = fs::read_to_string(
        "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors",
    )
    .await
    .context("read available governors")?;
    Ok(content.split_whitespace().map(String::from).collect())
}

/// Current governor for a specific CPU core.
///
/// # Errors
/// Returns error if sysfs path is unreadable.
pub async fn current(core: u32) -> Result<String> {
    let path = format!("/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_governor");
    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("read governor: {path}"))?;
    Ok(content.trim().to_owned())
}
