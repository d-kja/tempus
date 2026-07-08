#![allow(non_snake_case)]

use crate::bridge;
use crate::components::compact_timer::CompactTimer;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;

static CSS: Asset = asset!("/assets/app.css");

pub fn App() -> Element {
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
        div {
            class: "app-root",
            CompactTimer {}
        }
    }
}