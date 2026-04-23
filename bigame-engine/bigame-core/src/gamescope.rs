//! Gamescope micro-compositor configuration and launch management.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Gamescope display and rendering configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Enable FSR (`FidelityFX` Super Resolution).
    pub fsr: bool,
    /// FSR sharpness (0–20, lower = sharper).
    pub fsr_sharpness: u8,
    /// Framerate limit (0 = unlimited).
    pub framerate_limit: u32,
    /// Enable `MangoHud` overlay.
    pub mangohud: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fsr: false,
            fsr_sharpness: 5,
            framerate_limit: 0,
            mangohud: false,
        }
    }
}

impl Config {
    /// Build command-line arguments for gamescope launch.
    #[must_use]
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "-w".into(),
            self.width.to_string(),
            "-h".into(),
            self.height.to_string(),
        ];
        if self.fsr {
            args.push("--fsr".into());
            args.push("--fsr-sharpness".into());
            args.push(self.fsr_sharpness.to_string());
        }
        if self.framerate_limit > 0 {
            args.push("-r".into());
            args.push(self.framerate_limit.to_string());
        }
        if self.mangohud {
            args.push("--mangoapp".into());
        }
        args
    }

    /// Build a `Command` for launching gamescope with a child executable.
    ///
    /// `gamescope [config args] -- <command> [command_args...]`
    #[must_use]
    pub fn build_command(&self, command: &str, command_args: &[&str]) -> std::process::Command {
        let mut cmd = std::process::Command::new("gamescope");
        cmd.args(self.to_args());
        cmd.arg("--");
        cmd.arg(command);
        cmd.args(command_args);
        cmd
    }
}

/// Launch gamescope with the given config and child command.
///
/// Returns the spawned child process handle.
///
/// # Errors
/// Returns error if gamescope binary is not found or fails to spawn.
pub fn launch(
    config: &Config,
    command: &str,
    command_args: &[&str],
) -> Result<std::process::Child> {
    config
        .build_command(command, command_args)
        .spawn()
        .with_context(|| format!("spawn gamescope with: {command}"))
}

/// Global default Gamescope config path: `$XDG_CONFIG_HOME/bigame-mode/gamescope.toml`.
fn global_config_path() -> std::path::PathBuf {
    let config = std::env::var("XDG_CONFIG_HOME").map_or_else(
        |_| {
            std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
                .join(".config")
        },
        std::path::PathBuf::from,
    );
    config.join("bigame-mode").join("gamescope.toml")
}

/// Load global default Gamescope config from XDG config dir.
///
/// Returns `Config::default()` if the file does not exist or is unparseable.
#[must_use]
pub fn load_global() -> Config {
    let path = global_config_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

/// Save global default Gamescope config to XDG config dir.
///
/// # Errors
/// Returns error if the config directory or file cannot be written.
pub fn save_global(config: &Config) -> Result<()> {
    let path = global_config_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create config dir: {}", parent.display()))?;
    }
    let content = toml::to_string_pretty(config).context("serialize gamescope config")?;
    std::fs::write(&path, &content)
        .with_context(|| format!("write gamescope config: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_args() {
        let cfg = Config::default();
        let args = cfg.to_args();
        assert_eq!(args, vec!["-w", "1920", "-h", "1080"]);
    }

    #[test]
    fn fsr_enabled() {
        let cfg = Config {
            fsr: true,
            fsr_sharpness: 3,
            ..Config::default()
        };
        let args = cfg.to_args();
        assert!(args.contains(&"--fsr".to_owned()));
        assert!(args.contains(&"--fsr-sharpness".to_owned()));
        assert!(args.contains(&"3".to_owned()));
    }

    #[test]
    fn framerate_limit() {
        let cfg = Config {
            framerate_limit: 144,
            ..Config::default()
        };
        let args = cfg.to_args();
        assert!(args.contains(&"-r".to_owned()));
        assert!(args.contains(&"144".to_owned()));
    }

    #[test]
    fn framerate_zero_omitted() {
        let cfg = Config::default();
        let args = cfg.to_args();
        assert!(!args.contains(&"-r".to_owned()));
    }

    #[test]
    fn mangohud_flag() {
        let cfg = Config {
            mangohud: true,
            ..Config::default()
        };
        let args = cfg.to_args();
        assert!(args.contains(&"--mangoapp".to_owned()));
    }

    #[test]
    fn all_features() {
        let cfg = Config {
            width: 2560,
            height: 1440,
            fsr: true,
            fsr_sharpness: 0,
            framerate_limit: 60,
            mangohud: true,
        };
        let args = cfg.to_args();
        assert_eq!(
            args,
            vec![
                "-w",
                "2560",
                "-h",
                "1440",
                "--fsr",
                "--fsr-sharpness",
                "0",
                "-r",
                "60",
                "--mangoapp"
            ]
        );
    }

    #[test]
    fn serde_round_trip() {
        let cfg = Config {
            width: 3840,
            height: 2160,
            fsr: true,
            fsr_sharpness: 10,
            framerate_limit: 30,
            mangohud: false,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.width, 3840);
        assert_eq!(parsed.height, 2160);
        assert!(parsed.fsr);
        assert_eq!(parsed.fsr_sharpness, 10);
        assert_eq!(parsed.framerate_limit, 30);
        assert!(!parsed.mangohud);
    }

    #[test]
    fn build_command_basic() {
        let cfg = Config::default();
        let cmd = cfg.build_command("steam", &["-gamepadui"]);
        let prog = cmd.get_program().to_string_lossy().to_string();
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(prog, "gamescope");
        // ends with: -- steam -gamepadui
        assert!(args.contains(&"--".to_owned()));
        assert!(args.contains(&"steam".to_owned()));
        assert!(args.contains(&"-gamepadui".to_owned()));
    }

    #[test]
    fn save_load_global_round_trip() {
        // Redirect XDG_CONFIG_HOME to a temp dir so we don't pollute the user's config.
        let tmp = std::env::temp_dir().join(format!("bigame_gs_test_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        // SAFETY: single-threaded test; no other threads read XDG_CONFIG_HOME here.
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", &tmp);
        }

        let cfg = Config {
            width: 2560,
            height: 1440,
            fsr: true,
            fsr_sharpness: 3,
            framerate_limit: 144,
            mangohud: true,
        };
        save_global(&cfg).expect("save_global should succeed");
        let loaded = load_global();
        assert_eq!(loaded.width, 2560);
        assert_eq!(loaded.height, 1440);
        assert!(loaded.fsr);
        assert_eq!(loaded.fsr_sharpness, 3);
        assert_eq!(loaded.framerate_limit, 144);
        assert!(loaded.mangohud);

        // Cleanup
        std::fs::remove_dir_all(&tmp).ok();
        // SAFETY: restoring env to pre-test state.
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }

    #[test]
    fn load_global_missing_file_returns_default() {
        // SAFETY: single-threaded test; no other threads read XDG_CONFIG_HOME here.
        unsafe {
            std::env::set_var("XDG_CONFIG_HOME", "/tmp/bigame_gs_nonexistent_99999");
        }
        let cfg = load_global();
        assert_eq!(cfg.width, 1920);
        assert_eq!(cfg.height, 1080);
        assert!(!cfg.fsr);
        // SAFETY: restoring env to pre-test state.
        unsafe {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
    }
}
