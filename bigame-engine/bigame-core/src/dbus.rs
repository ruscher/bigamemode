//! D-Bus integration with `GameMode` and `PowerProfiles` daemons.

#![allow(clippy::missing_errors_doc)]

use std::sync::OnceLock;

/// Cached system bus connection (for `PowerProfiles`).
static SYSTEM_CONN: OnceLock<Option<zbus::blocking::Connection>> = OnceLock::new();

/// Cached session bus connection (for `GameMode`).
static SESSION_CONN: OnceLock<Option<zbus::blocking::Connection>> = OnceLock::new();

fn session_conn() -> Option<&'static zbus::blocking::Connection> {
    SESSION_CONN
        .get_or_init(|| zbus::blocking::Connection::session().ok())
        .as_ref()
}

fn system_conn() -> Option<&'static zbus::blocking::Connection> {
    SYSTEM_CONN
        .get_or_init(|| zbus::blocking::Connection::system().ok())
        .as_ref()
}

// ── GameMode ────────────────────────────────────────────────────────────────

/// Proxy for the `GameMode` D-Bus interface.
///
/// `GameMode` provides system-wide performance optimizations
/// when games are running (Feral Interactive).
#[zbus::proxy(
    interface = "com.feralinteractive.GameMode",
    default_service = "com.feralinteractive.GameMode",
    default_path = "/com/feralinteractive/GameMode"
)]
trait GameMode {
    /// Register a game process for optimization.
    fn register_game(&self, pid: i32) -> zbus::Result<i32>;

    /// Unregister a game process.
    fn unregister_game(&self, pid: i32) -> zbus::Result<i32>;

    /// Query number of active registered games.
    fn query_status(&self) -> zbus::Result<i32>;
}

/// Query `GameMode` active game count (synchronous / blocking).
///
/// Caches the D-Bus session connection for reuse across calls.
/// Returns 0 if `GameMode` daemon is unavailable.
#[must_use] 
pub fn gamemode_active_count() -> i32 {
    let Some(conn) = session_conn() else {
        return 0;
    };
    GameModeProxyBlocking::new(conn)
        .ok()
        .and_then(|p| p.query_status().ok())
        .unwrap_or(0)
}

/// Check if falcond service is running by looking for its status file.
#[must_use]
pub fn falcond_is_running() -> bool {
    std::path::Path::new(crate::status::STATUS_PATH).exists()
}


/// Register the current process in `GameMode` for performance optimizations.
///
/// Returns `true` if registration succeeded.
#[must_use]
pub fn gamemode_register() -> bool {
    let Some(conn) = session_conn() else {
        return false;
    };
    #[allow(clippy::cast_possible_wrap)]
    let pid = std::process::id() as i32;
    GameModeProxyBlocking::new(conn)
        .ok()
        .and_then(|p| p.register_game(pid).ok())
        .is_some_and(|r| r == 0)
}

/// Unregister the current process from `GameMode`.
///
/// Returns `true` if unregistration succeeded.
#[must_use]
pub fn gamemode_unregister() -> bool {
    let Some(conn) = session_conn() else {
        return false;
    };
    #[allow(clippy::cast_possible_wrap)]
    let pid = std::process::id() as i32;
    GameModeProxyBlocking::new(conn)
        .ok()
        .and_then(|p| p.unregister_game(pid).ok())
        .is_some_and(|r| r == 0)
}

// ── PowerProfiles ───────────────────────────────────────────────────────────

/// Proxy for `net.hadess.PowerProfiles` (power-profiles-daemon).
#[zbus::proxy(
    interface = "net.hadess.PowerProfiles",
    default_service = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
trait PowerProfiles {
    /// Current active power profile.
    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;

    /// Set the active power profile.
    #[zbus(property)]
    fn set_active_profile(&self, profile: &str) -> zbus::Result<()>;
}

/// Get current power profile (blocking).
///
/// Returns "balanced", "performance", or "power-saver".
/// Returns `None` if daemon is unavailable.
#[must_use] 
pub fn power_profile_get() -> Option<String> {
    let conn = system_conn()?;
    PowerProfilesProxyBlocking::new(conn)
        .ok()
        .and_then(|p| p.active_profile().ok())
}

/// Set power profile (blocking).
///
/// Valid values: "balanced", "performance", "power-saver".
#[must_use] 
pub fn power_profile_set(profile: &str) -> bool {
    let Some(conn) = system_conn() else {
        return false;
    };
    PowerProfilesProxyBlocking::new(conn)
        .ok()
        .and_then(|p| p.set_active_profile(profile).ok())
        .is_some()
}

// ── Falcond status D-Bus service ────────────────────────────────────────────

/// D-Bus service that broadcasts `falcond` status changes on the session bus.
///
/// Bus name   : `com.biglinux.BiGameMode1` (session)
/// Object path: `/com/biglinux/BiGameMode/Falcond`
/// Interface  : `com.biglinux.BiGameMode.Falcond`
///
/// External tools (scripts, shell widgets, monitors) can subscribe to the
/// `StatusChanged` signal instead of polling `/tmp/falcond_status` directly.
pub mod service {
    use zbus::object_server::SignalEmitter;

    const BUS_NAME: &str = "com.biglinux.BiGameMode1";
    const OBJECT_PATH: &str = "/com/biglinux/BiGameMode/Falcond";
    /// How often the service polls the status file for changes.
    const POLL_MS: u64 = 500;

    struct FalcondIface;

    #[zbus::interface(name = "com.biglinux.BiGameMode.Falcond")]
    impl FalcondIface {
        /// Return current falcond status (raw key-value text).
        #[allow(clippy::unused_self)] // zbus interface methods require &self
        fn get_status(&self) -> String {
            std::fs::read_to_string(crate::status::STATUS_PATH).unwrap_or_default()
        }

        /// Emitted whenever `/tmp/falcond_status` changes.
        #[zbus(signal)]
        async fn status_changed(ctx: &SignalEmitter<'_>, content: &str) -> zbus::Result<()>;
    }

    /// Async service loop: register on the session bus, poll the file, emit signals.
    async fn run() -> zbus::Result<()> {
        let conn = zbus::connection::Builder::session()?
            .name(BUS_NAME)?
            .serve_at(OBJECT_PATH, FalcondIface)?
            .build()
            .await?;

        tracing::info!("falcond D-Bus status service registered as {BUS_NAME}");

        let mut last = String::new();
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(POLL_MS)).await;

            let Ok(content) = tokio::fs::read_to_string(crate::status::STATUS_PATH).await else {
                continue;
            };

            if content == last {
                continue;
            }
            last = content.clone();

            let iface_ref = conn
                .object_server()
                .interface::<_, FalcondIface>(OBJECT_PATH)
                .await;
            let Ok(iface) = iface_ref else { continue; };
            FalcondIface::status_changed(iface.signal_emitter(), &content)
                .await
                .ok();
        }
    }

    /// Spawn the falcond D-Bus status service in a background thread.
    ///
    /// Non-blocking: returns immediately. The service runs for the process lifetime.
    /// Silently exits if the session bus is unavailable (headless/tty environments).
    pub fn start() {
        std::thread::Builder::new()
            .name("bigame-dbus-service".into())
            .spawn(|| {
                let rt = match tokio::runtime::Runtime::new() {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("D-Bus service: failed to create tokio runtime: {e}");
                        return;
                    }
                };
                rt.block_on(async {
                    if let Err(e) = run().await {
                        // Not an error in headless/tty environments where no session bus exists.
                        tracing::debug!("falcond D-Bus status service exited: {e}");
                    }
                });
            })
            .ok(); // silently ignore thread spawn failure
    }
}
