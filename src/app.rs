#![allow(non_snake_case)]

use crate::bridge;
use crate::components::compact_timer::CompactTimer;
use crate::full_app_window::FullAppWindow;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/app.css");

fn use_is_settings_window() -> bool {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| s.contains("settings"))
        .unwrap_or(false)
}

pub fn App() -> Element {
    let is_settings = use_is_settings_window();

    if is_settings {
        rsx! {
            document::Link { rel: "stylesheet", href: CSS }
            FullAppWindow {}
        }
    } else {
        let mut state = use_context_provider(|| AppState::new());

        use_effect({
            let mut timer = state.timer;
            move || {
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(active) = bridge::get_active_entry().await {
                        if let Some(entry) = active {
                            timer.set(TimerState::Running(entry));
                        }
                    }
                });
            }
        });

        rsx! {
            document::Link { rel: "stylesheet", href: CSS }
            div { class: "app-root", CompactTimer {} }
        }
    }
}
