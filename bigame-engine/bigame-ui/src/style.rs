//! CSS theme loader + `GResource` registration for BiGame-mode.

use gtk4::{gio, glib};

/// Register compiled `GResource` bundle and load application CSS.
///
/// Call once during `Application::connect_startup`.
///
/// # Panics
/// Panics if the compiled `GResource` cannot be loaded (build.rs failure).
pub fn load_css() {
    // Register compiled GResource from build.rs output
    let bytes = glib::Bytes::from_static(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/resources.gresource"
    )));
    let resource = gio::Resource::from_data(&bytes).expect("load gresource");
    gio::resources_register(&resource);

    // Load CSS from the registered resource
    let provider = gtk4::CssProvider::new();
    provider.load_from_resource("/com/biglinux/BiGameMode/style.css");

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
