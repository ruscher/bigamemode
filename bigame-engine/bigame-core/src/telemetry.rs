//! System telemetry: CPU/GPU frequency, temperature, utilization from sysfs.

use anyhow::{Context, Result};
use tokio::fs;

/// CPU core telemetry snapshot.
#[derive(Debug, Clone)]
pub struct CpuSnapshot {
    /// Current frequency in MHz.
    pub freq_mhz: u64,
    /// Active governor.
    pub governor: String,
}

/// GPU telemetry snapshot.
#[derive(Debug, Clone)]
pub struct GpuSnapshot {
    /// Current GPU clock in MHz (if available).
    pub freq_mhz: Option<u64>,
    /// GPU temperature in °C (if available).
    pub temp_celsius: Option<f64>,
}

/// Read current CPU frequency for a given core.
///
/// # Errors
/// Returns error if sysfs path is unreadable or contains invalid data.
pub async fn cpu_freq_mhz(core: u32) -> Result<u64> {
    let path = format!("/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_cur_freq");
    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("read CPU freq: {path}"))?;
    let khz: u64 = content.trim().parse().context("parse CPU freq")?;
    Ok(khz / 1000)
}

/// Read current CPU governor for a given core.
///
/// # Errors
/// Returns error if sysfs path is unreadable.
pub async fn cpu_governor(core: u32) -> Result<String> {
    let path = format!("/sys/devices/system/cpu/cpu{core}/cpufreq/scaling_governor");
    let content = fs::read_to_string(&path)
        .await
        .with_context(|| format!("read governor: {path}"))?;
    Ok(content.trim().to_owned())
}

/// Collect full CPU snapshot for a given core.
///
/// # Errors
/// Returns error if any sysfs read fails.
pub async fn cpu_snapshot(core: u32) -> Result<CpuSnapshot> {
    let (freq, gov) = tokio::try_join!(cpu_freq_mhz(core), cpu_governor(core))?;
    Ok(CpuSnapshot {
        freq_mhz: freq,
        governor: gov,
    })
}

// ── GPU Telemetry ────────────────────────────────────────────────────────────

/// Collect a GPU telemetry snapshot.
///
/// Tries AMD sysfs first, then NVIDIA `nvidia-smi`.
/// Returns `None` fields if the GPU is not readable.
pub async fn gpu_snapshot() -> GpuSnapshot {
    GpuSnapshot {
        freq_mhz: gpu_freq_mhz().await.ok(),
        temp_celsius: gpu_temp_celsius().await.ok(),
    }
}

/// Read GPU core clock frequency in MHz.
///
/// AMD path: `/sys/class/drm/card*/device/hwmon/hwmon*/freq1_input` (Hz → MHz).
/// NVIDIA fallback: `nvidia-smi --query-gpu=clocks.current.graphics`.
async fn gpu_freq_mhz() -> Result<u64> {
    // AMD: iterate card* dirs looking for hwmon/hwmon*/freq1_input
    if let Some(val) = amd_hwmon_read_u64("freq1_input").await {
        return Ok(val / 1_000_000); // Hz → MHz
    }
    // NVIDIA fallback
    nvidia_smi_query("clocks.current.graphics")
        .await
        .and_then(|s| s.trim().parse::<u64>().context("parse NVIDIA freq"))
}

/// Read GPU temperature in °C.
///
/// AMD path: `/sys/class/drm/card*/device/hwmon/hwmon*/temp1_input` (m°C → °C).
/// NVIDIA fallback: `nvidia-smi --query-gpu=temperature.gpu`.
async fn gpu_temp_celsius() -> Result<f64> {
    if let Some(val) = amd_hwmon_read_u64("temp1_input").await {
        // millidegrees → °C; u64 fits exactly in f64 for sensor ranges.
        #[allow(clippy::cast_precision_loss)]
        return Ok(val as f64 / 1000.0);
    }
    nvidia_smi_query("temperature.gpu")
        .await
        .and_then(|s| s.trim().parse::<f64>().context("parse NVIDIA temp"))
}

/// Walk `/sys/class/drm/card*/device/hwmon/hwmon*/` and return the first
/// readable integer value for `filename`.
async fn amd_hwmon_read_u64(filename: &str) -> Option<u64> {
    let drm_dir = std::fs::read_dir("/sys/class/drm").ok()?;
    for card_entry in drm_dir.flatten() {
        let hwmon_base = card_entry.path().join("device/hwmon");
        let hwmon_dir = std::fs::read_dir(&hwmon_base).ok()?;
        for hwmon_entry in hwmon_dir.flatten() {
            let path = hwmon_entry.path().join(filename);
            if let Ok(content) = fs::read_to_string(&path).await {
                if let Ok(val) = content.trim().parse::<u64>() {
                    return Some(val);
                }
            }
        }
    }
    None
}

/// Run `nvidia-smi --query-gpu=<field> --format=csv,noheader,nounits`.
///
/// Returns the trimmed stdout string, or an error if nvidia-smi is not present
/// or the command fails.
async fn nvidia_smi_query(field: &str) -> Result<String> {
    let output = tokio::process::Command::new("nvidia-smi")
        .args(["--query-gpu", field, "--format=csv,noheader,nounits"])
        .output()
        .await
        .context("spawn nvidia-smi")?;
    anyhow::ensure!(output.status.success(), "nvidia-smi exited with error");
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_snapshot_clone() {
        let snap = CpuSnapshot { freq_mhz: 3600, governor: "performance".into() };
        let cloned = snap.clone();
        assert_eq!(cloned.freq_mhz, 3600);
        assert_eq!(cloned.governor, "performance");
    }

    #[test]
    fn gpu_snapshot_clone() {
        let snap = GpuSnapshot { freq_mhz: Some(1800), temp_celsius: Some(72.5) };
        let cloned = snap.clone();
        assert_eq!(cloned.freq_mhz, Some(1800));
        assert_eq!(cloned.temp_celsius, Some(72.5));
    }

    #[test]
    fn gpu_snapshot_none_fields() {
        let snap = GpuSnapshot { freq_mhz: None, temp_celsius: None };
        let cloned = snap.clone();
        assert!(cloned.freq_mhz.is_none());
        assert!(cloned.temp_celsius.is_none());
    }

    // Error paths — nonexistent CPU core → sysfs read fails

    #[tokio::test]
    async fn cpu_freq_nonexistent_core_errors() {
        assert!(cpu_freq_mhz(99999).await.is_err());
    }

    #[tokio::test]
    async fn cpu_governor_nonexistent_core_errors() {
        assert!(cpu_governor(99999).await.is_err());
    }

    #[tokio::test]
    async fn cpu_snapshot_nonexistent_core_errors() {
        assert!(cpu_snapshot(99999).await.is_err());
    }
}
