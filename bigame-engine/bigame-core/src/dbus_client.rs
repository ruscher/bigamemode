use zbus::proxy;

/// Proxy trait representing the BiGameMode Root Daemon interface.
#[proxy(
    interface = "com.biglinux.BiGameMode",
    default_service = "com.biglinux.BiGameMode",
    default_path = "/com/biglinux/BiGameMode"
)]
pub trait BiGameDaemon {
    async fn save_profile(&self, name: &str, json_payload: &str) -> zbus::Result<()>;
    async fn delete_profile(&self, name: &str) -> zbus::Result<()>;
    async fn apply_falcond_config(&self, config_payload: &str) -> zbus::Result<()>;
    async fn set_vcache_mode(&self, mode: &str) -> zbus::Result<()>;
    async fn set_cpu_governor(&self, governor: &str) -> zbus::Result<()>;
    async fn ping(&self) -> zbus::Result<String>;
}

/// Helper function to easily get a connected proxy.
pub async fn daemon_proxy() -> anyhow::Result<BiGameDaemonProxy<'static>> {
    let connection = zbus::Connection::system().await?;
    let proxy = BiGameDaemonProxy::new(&connection).await?;
    Ok(proxy)
}
