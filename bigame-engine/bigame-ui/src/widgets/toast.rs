//! Toast notification helpers.

use libadwaita as adw;
use adw::prelude::*;

/// Find the nearest `AdwToastOverlay` ancestor of a widget.
pub fn find_overlay(widget: &impl IsA<gtk4::Widget>) -> Option<adw::ToastOverlay> {
    let mut current = widget.ancestor(adw::ToastOverlay::static_type());
    while let Some(w) = current {
        if let Ok(overlay) = w.clone().downcast::<adw::ToastOverlay>() {
            return Some(overlay);
        }
        current = w.ancestor(adw::ToastOverlay::static_type());
    }
    None
}

/// Show a toast message by searching for the nearest `ToastOverlay`.
pub fn show(widget: &impl IsA<gtk4::Widget>, message: &str) {
    if let Some(overlay) = find_overlay(widget) {
        overlay.add_toast(adw::Toast::new(message));
    }
}
