mod app;
mod bridge;
mod components;
mod settings_window;
mod state;

use app::App;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use settings_window::SettingsWindow;

fn location_has_settings() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .map(|h| h.contains("settings"))
        .unwrap_or(false)
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    if location_has_settings() {
        launch(SettingsWindow);
    } else {
        launch(App);
    }
}