//! BiGame-mode Libadwaita application entry point.

mod app;
pub mod i18n;
pub mod settings;
mod style;
mod tray;
mod views;
mod widgets;
mod window;

fn main() -> libadwaita::glib::ExitCode {
    i18n::init();
    app::run()
}
