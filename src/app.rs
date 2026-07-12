#![allow(non_snake_case)]

use crate::bridge;
use crate::components::compact_timer::CompactTimer;
use crate::components::edit_entry_window::EditEntryWindow;
use crate::components::new_entry_window::NewEntryWindow;
use crate::full_app_window::FullAppWindow;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/app.css");

fn use_page() -> &'static str {
    web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| {
            if s.contains("settings") {
                "settings"
            } else if s.contains("edit-entry") {
                "edit-entry"
            } else if s.contains("new-entry") {
                "new-entry"
            } else {
                "compact"
            }
        })
        .unwrap_or("compact")
}

pub fn App() -> Element {
    match use_page() {
        "settings" => {
            rsx! {
                document::Link { rel: "stylesheet", href: CSS }
                FullAppWindow {}
            }
        }
        "new-entry" => {
            rsx! {
                NewEntryWindow {}
            }
        }
        "edit-entry" => {
            rsx! {
                EditEntryWindow {}
            }
        }
        _ => {
            let state = use_context_provider(|| AppState::new());

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
}
