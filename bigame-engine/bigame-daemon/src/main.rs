use anyhow::Result;
use bigame_core::profiles::USER_PROFILES_DIR;
use std::fs;
use std::path::Path;
use tokio::process::Command;
use tracing::{info, error, Level};
use tracing_subscriber::FmtSubscriber;
use zbus::{connection, interface};

struct BiGameDaemon;

#[interface(name = "com.biglinux.BiGameMode")]
impl BiGameDaemon {
    /// Save profile JSON payload directly to /usr/share/falcond/profiles/user/
    /// The daemon parses, validates, and writes safely as root.
    async fn save_profile(&self, name: String, json_payload: String) -> Result<(), zbus::fdo::Error> {
        info!("D-Bus Request: Save profile '{}'", name);
        
        let target_dir = Path::new(USER_PROFILES_DIR);
        if !target_dir.exists() {
            if let Err(e) = fs::create_dir_all(target_dir) {
                error!("Failed to create profiles directory: {}", e);
                return Err(zbus::fdo::Error::Failed(format!("mkdir auth err: {}", e)));
            }
        }

        let file_path = target_dir.join(format!("{}.conf", name));
        if let Err(e) = fs::write(&file_path, json_payload) {
            error!("Failed to write profile {}: {}", name, e);
            return Err(zbus::fdo::Error::Failed(format!("write err: {}", e)));
        }

        info!("Profile '{}' saved successfully at {:?}", name, file_path);
        Ok(())
    }

    /// Delete user profile by name
    async fn delete_profile(&self, name: String) -> Result<(), zbus::fdo::Error> {
        info!("D-Bus Request: Delete profile '{}'", name);
        let file_path = Path::new(USER_PROFILES_DIR).join(format!("{}.conf", name));
        
        if file_path.exists() {
            if let Err(e) = fs::remove_file(&file_path) {
                error!("Failed to delete profile {}: {}", name, e);
                return Err(zbus::fdo::Error::Failed(format!("rm err: {}", e)));
            }
            info!("Profile '{}' deleted.", name);
        } else {
            info!("Profile '{}' not found, ignoring delete.", name);
        }
        Ok(())
    }

    /// Apply falcond configuration and send SIGHUP
    async fn apply_falcond_config(&self, config_payload: String) -> Result<(), zbus::fdo::Error> {
        info!("D-Bus Request: Apply falcond global config");
        let conf_path = "/etc/falcond/falcond.conf";
        
        if let Err(e) = fs::write(conf_path, config_payload) {
            error!("Failed to write {}: {}", conf_path, e);
            return Err(zbus::fdo::Error::Failed(format!("write config err: {}", e)));
        }

        // Restart falcond or SIGHUP
        info!("Sending SIGHUP to falcond daemon...");
        let status = Command::new("pkill")
            .arg("-HUP")
            .arg("falcond")
            .status()
            .await;

        match status {
            Ok(s) if s.success() => info!("falcond successfully reloaded."),
            Ok(s) => error!("pkill returned non-zero status: {}", s),
            Err(e) => error!("failed to execute pkill: {}", e),
        }

        Ok(())
    }

    /// Write vcache governor to sysfs
    async fn set_vcache_mode(&self, mode: String) -> Result<(), zbus::fdo::Error> {
        info!("D-Bus Request: Set vcache mode to '{}'", mode);
        let sysfs_path = "/sys/bus/platform/drivers/amd_x3d_vcache/AMDI0015:00/amd_x3d_mode";
        
        if Path::new(sysfs_path).exists() {
            if let Err(e) = fs::write(sysfs_path, mode) {
                error!("Failed to write vcache sysfs: {}", e);
                return Err(zbus::fdo::Error::Failed(format!("vcache err: {}", e)));
            }
            info!("vcache sysfs successfully updated.");
        } else {
            error!("vcache sysfs path not found. Target not supported.");
            return Err(zbus::fdo::Error::NotSupported("Hardware does not support dynamic x3d vcache".into()));
        }

        Ok(())
    }

    /// Ping
    async fn ping(&self) -> Result<String, zbus::fdo::Error> {
        Ok("pong from root bigame-daemon".to_string())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting bigame-daemon (root orchestrator)...");

    let daemon = BiGameDaemon;

    let _conn = connection::Builder::system()?
        .name("com.biglinux.BiGameMode")?
        .serve_at("/com/biglinux/BiGameMode", daemon)?
        .build()
        .await?;

    info!("D-Bus system connection established on com.biglinux.BiGameMode.");
    info!("Looping indefinitely to serve requests...");

    // Run forever
    std::future::pending::<()>().await;

    Ok(())
}
