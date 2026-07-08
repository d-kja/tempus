mod app;
mod bridge;
mod components;
mod full_app_window;
mod state;

use app::App;
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;
use full_app_window::FullAppWindow;

fn location_has_settings() -> bool {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .map(|h| h.contains("settings"))
        .unwrap_or(false)
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    if location_has_settings() {
        launch(FullAppWindow);
    } else {
        launch(App);
    }
}