//! Application setup and lifecycle.
//!
//! Supports background daemon mode: closing the window hides it
//! instead of quitting. Re-activate to show again.

use libadwaita as adw;
use adw::prelude::*;
use gtk4::glib;

use crate::i18n::i18n;
use crate::style;
use crate::tray;
use crate::window;

/// Reverse-domain application identifier.
const APP_ID: &str = "com.biglinux.BiGameMode";

/// Create, configure, and run the BiGame-mode application.
///
/// The app stays alive in the background after the window is closed.
/// Re-activating (e.g. via desktop file) will re-present the window.
pub fn run() -> adw::glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_startup(|app| {
        style::load_css();
        // Keep app alive even when all windows are closed.
        // Intentionally leak the guard — app should never release.
        std::mem::forget(app.hold());

        // Falcond D-Bus status service: re-broadcasts /tmp/falcond_status as D-Bus signal.
        // External tools subscribe to com.biglinux.BiGameMode1 instead of polling the file.
        bigame_core::dbus::service::start();

        // Quit action for explicit exit
        let quit = adw::gio::ActionEntry::builder("quit")
            .activate(|app: &adw::Application, _, _| app.quit())
            .build();

        // About dialog action
        let about = adw::gio::ActionEntry::builder("about")
            .activate(|app: &adw::Application, _, _| {
                show_about_dialog(app);
            })
            .build();

        app.add_action_entries([quit, about]);

        // Keyboard shortcuts
        app.set_accels_for_action("app.quit", &["<Control>q"]);
    });

    app.connect_activate(|app| {
        // Re-present existing window or build new one
        if let Some(win) = app.active_window() {
            win.present();
        } else {
            let (win, error_indicator) = window::build(app);
            // Hide on close instead of destroying
            win.connect_close_request(|w| {
                w.set_visible(false);
                glib::Propagation::Stop
            });
            win.present();

            // System tray — poll actions from GTK main loop
            let (tray_handle, tray_rx) = tray::spawn();
            let app_ref = app.clone();
            
            // Poll tray actions
            glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
                while let Ok(action) = tray_rx.try_recv() {
                    match action {
                        tray::TrayAction::Activate => {
                            if let Some(w) = app_ref.active_window() {
                                w.set_visible(true);
                                w.present();
                            }
                        }
                        tray::TrayAction::Quit => {
                            app_ref.quit();
                        }
                        tray::TrayAction::SwitchProfile(name) => {
                            tracing::info!("Tray: switching to profile '{name}'");
                            let _ = bigame_core::dbus::gamemode_register();
                        }
                    }
                }
                glib::ControlFlow::Continue
            });

            // Update tray status periodically
            let th_ref = tray_handle.clone();
            glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
                let gm_active = bigame_core::dbus::gamemode_active_count() > 0;
                let falcond_running = bigame_core::dbus::falcond_is_running();
                
                let status = if !falcond_running {
                    error_indicator.set_error_with_action(
                        &i18n("Service Not Running or Crashed"),
                        &i18n("BiGameMode background daemon (falcond) is not running.\nThis can happen if the configuration files got corrupted by an older version of the UI."),
                        &i18n("Click 'Repair & Enable' to automatically reset corrupted configurations and start the service."),
                        &i18n("Repair & Enable"),
                        vec![
                            "pkexec".into(),
                            "sh".into(),
                            "-c".into(),
                            "rm -f /etc/falcond/config.conf; rm -f /usr/share/falcond/profiles/user/*.conf; systemctl enable --now falcond".into(),
                        ],
                    );
                    tray::Status::Warning
                } else {
                    error_indicator.clear();
                    if gm_active {
                        tray::Status::Active
                    } else {
                        tray::Status::Idle
                    }
                };
                
                th_ref.set_status(status);
                glib::ControlFlow::Continue
            });
        }
    });

    app.run()
}

/// Present the About dialog with system information.
fn show_about_dialog(app: &adw::Application) {
    let sys_info = collect_system_info();

    let dialog = adw::AboutDialog::builder()
        .application_name("BiGame-mode")
        .application_icon(APP_ID)
        .version(env!("CARGO_PKG_VERSION"))
        .developer_name("Rafael Ruscher")
        .website("https://github.com/biglinux/bigamemode")
        .issue_url("https://github.com/biglinux/bigamemode/issues")
        .license_type(gtk4::License::Gpl30)
        .comments(i18n("Performance tuning for Linux gaming"))
        .debug_info(&sys_info)
        .debug_info_filename("bigame-mode-debug.txt")
        .build();

    dialog.add_credit_section(Some(&i18n("Developers")), &["Rafael Ruscher <rruscher@gmail.com>"]);
    dialog.add_credit_section(Some(&i18n("Special Thanks")), &["Barnabé di Kartola", "Alessandro (System Infotech)", "Pacheco (System Infotech)"]);

    if let Some(win) = app.active_window() {
        dialog.present(Some(&win));
    }
}

/// Collect system information for the About dialog debug section.
fn collect_system_info() -> String {
    let mut lines = Vec::new();

    lines.push(format!("BiGame-mode {}", env!("CARGO_PKG_VERSION")));
    lines.push(String::new());

    // Kernel
    if let Ok(kernel) = std::fs::read_to_string("/proc/version") {
        if let Some(first) = kernel.lines().next() {
            lines.push(format!("Kernel: {first}"));
        }
    }

    // CPU model
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if let Some(model) = line.strip_prefix("model name") {
                if let Some(val) = model.split_once(':').map(|(_, v)| v.trim()) {
                    lines.push(format!("CPU: {val}"));
                    break;
                }
            }
        }
    }

    // GPU (DRI device)
    if let Ok(output) = std::process::Command::new("lspci").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("VGA") || line.contains("3D controller") {
                lines.push(format!("GPU: {line}"));
                break;
            }
        }
    }

    // Installed sched-ext schedulers
    let scheds = bigame_core::sched::detect_installed();
    lines.push(format!("Schedulers: {}", scheds.join(", ")));

    // GameMode status
    let gm = bigame_core::dbus::gamemode_active_count();
    lines.push(format!("GameMode active: {gm}"));

    // Power profile
    if let Some(pp) = bigame_core::dbus::power_profile_get() {
        lines.push(format!("Power profile: {pp}"));
    }

    lines.join("\n")
}
