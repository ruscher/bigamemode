//! Context-aware tutorial dialogs explaining each view's purpose and options.

use adw::prelude::*;
use libadwaita as adw;

use crate::i18n::i18n;

/// Show a tutorial dialog for the currently active tab.
///
/// `tab_name` — matches the `AdwViewStack` child name:
/// `"dashboard"` | `"profiles"` | `"tuning"` | `"gamescope"` | `"logs"` | `"settings"`.
pub fn show(widget: &impl IsA<gtk4::Widget>, tab_name: &str) {
    let (heading, body) = content(tab_name);

    let dialog = adw::AlertDialog::builder()
        .heading(heading)
        .body(body)
        .body_use_markup(true)
        .close_response("close")
        .default_response("close")
        .build();

    dialog.add_response("close", &i18n("Got It"));

    // Present attached to the top-level window so it's modal
    let win = widget.root().and_downcast::<gtk4::Window>();
    dialog.present(win.as_ref());
}

/// Return `(heading, body_markup)` for a given tab name.
fn content(tab: &str) -> (String, String) {
    match tab {
        "dashboard" => (
            i18n("Dashboard"),
            [
                i18n("<b>Stat Bar</b> — CPU frequency, GPU temperature and RAM at a glance."),
                i18n("<b>Performance Booster</b> — Enable or disable falcond for gaming optimisations."),
                i18n("<b>Frame Generation (lsfg-vk)</b> — Shows whether the Vulkan implicit layer is installed. No Steam launch options are needed — it activates automatically for configured games."),
                i18n("<b>Power Profile</b> — Current system power profile (Balanced / Performance / Power Saver)."),
                i18n("<b>Falcond Daemon</b> — Shows the active SCX scheduler, VCache mode and current game profile."),
                i18n("<b>System Telemetry</b> — Live sparkline graphs: CPU/GPU frequency, GPU temperature, disk I/O, network latency and RAM usage."),
                i18n("<b>Detected Games</b> — Games found via Steam, Lutris and Heroic. Create a performance profile or launch directly with Gamescope."),
            ]
            .join("\n\n"),
        ),

        "profiles" => (
            i18n("Game Profiles"),
            [
                i18n("<b>Profile Wizard</b> — Guided setup with presets optimised for competitive FPS, strategy, simulation and more."),
                i18n("<b>Per-game Profiles</b> — Override global Tuning settings for a specific game executable."),
                i18n("<b>Auto-Apply</b> — Falcond detects the game process at launch and applies the matching profile automatically."),
                i18n("<b>New Profile (+)</b> — Create a blank profile and customise every parameter manually."),
                i18n("<b>Import</b> — Load a .conf or .toml profile from disk. Drag-and-drop onto the list also works."),
            ]
            .join("\n\n"),
        ),

        "tuning" => (
            i18n("Tuning"),
            [
                i18n("<b>Falcond Daemon</b> — Start or stop the daemon and toggle auto-start on login."),
                i18n("<b>SCX Scheduler</b> — sched-ext BPF CPU scheduler:\n  · bpfland — Low latency, ideal for competitive games\n  · lavd — Load-aware, good for mixed workloads\n  · rusty — General-purpose, stable default\n  · flash — Maximum throughput"),
                i18n("<b>CPU Governor</b> — Performance keeps frequency at maximum; Powersave scales down to save energy."),
                i18n("<b>VCache Mode</b> — AMD Ryzen X3D only. Game Mode routes workloads to the VCache chiplet for lower memory latency."),
                i18n("<b>Frame Generation</b> — lsfg-vk Vulkan layer: inserts interpolated frames to boost perceived FPS.\n  · <b>Multiplier</b> (2–20): frames generated per real frame. Min 2 required by lsfg-vk.\n  · <b>Flow Scale</b> (25–100%): motion estimation resolution. Lower = faster; higher = quality.\n  · <b>Performance Mode</b>: lighter model with minor quality trade-off.\n  · <b>Lossless.dll Path</b>: optional custom DLL location (global/Tuning tab only)."),
                i18n("<b>Device Mode</b> — Optimise kernel tuning for Gaming or Desktop workloads."),
            ]
            .join("\n\n"),
        ),

        "gamescope" => (
            i18n("Gamescope"),
            [
                i18n("<b>Resolution</b> — Set the game render resolution independently from your desktop resolution."),
                i18n("<b>Framerate Limit</b> — Hard cap on FPS for consistent frame pacing."),
                i18n("<b>FSR</b> — FidelityFX Super Resolution upscaling. Sharpness: 0 = blurry · 20 = sharpest."),
                i18n("<b>MangoHud</b> — Enable the MangoHud performance overlay inside the Gamescope window."),
                i18n("<b>Command</b> — Game or program to launch inside Gamescope (e.g. <tt>steam -gamepadui</tt>)."),
                i18n("<b>Launch</b> — Applies current settings and starts the Gamescope compositor."),
            ]
            .join("\n\n"),
        ),

        "logs" => (
            i18n("Logs"),
            [
                i18n("<b>Falcond Status</b> — Live read of <tt>/tmp/falcond_status</tt>: current scheduler, governor and active profile."),
                i18n("<b>Gaming Services Journal</b> — journald entries from falcond and related gaming services."),
                i18n("<b>Kernel &amp; GPU Messages</b> — dmesg output filtered for GPU and gaming-related entries."),
                i18n("<b>Application Log</b> — BiGame-mode internal events: profile loads, D-Bus calls and errors."),
                i18n("<b>Auto-Refresh</b> — Logs update every 5 seconds. Click ↺ to force an immediate refresh."),
            ]
            .join("\n\n"),
        ),

        "settings" => (
            i18n("Settings"),
            [
                i18n("<b>Dark Mode</b> — Force dark colour scheme regardless of the system theme."),
                i18n("<b>Game Notifications</b> — Show desktop notifications when a game launches or exits."),
                i18n("<b>Ping Target</b> — Hostname or IP used by the Dashboard latency sparkline (default: 8.8.8.8)."),
                i18n("<b>About</b> — App version, author and source code information."),
            ]
            .join("\n\n"),
        ),

        _ => (
            i18n("Help"),
            i18n("No help available for this view."),
        ),
    }
}
