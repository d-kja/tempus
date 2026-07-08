#![allow(non_snake_case)]

use crate::bridge;
use crate::components::compact_timer::CompactTimer;
use crate::components::expanded_view::ExpandedView;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;

pub fn App() -> Element {
    let mut state = use_context_provider(|| AppState::new());

    use_effect({
        let mut timer = state.timer;
        let mut entries = state.entries;
        let mut projects = state.projects;
        let mut settings = state.settings;
        move || {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(active) = bridge::get_active_entry().await {
                    if let Some(entry) = active {
                        timer.set(TimerState::Running(entry));
                    }
                }
                if let Ok(e) = bridge::get_entries(Some(20), None).await {
                    entries.set(e);
                }
                if let Ok(p) = bridge::get_projects().await {
                    projects.set(p);
                }
                if let Ok(s) = bridge::get_settings().await {
                    settings.set(s);
                }
            });
        }
    });

    rsx! {
        div {
            class: "floating-panel h-screen w-screen overflow-hidden text-zinc-100 border border-zinc-800/60 flex flex-col",
            if *state.is_expanded.read() {
                ExpandedView {}
            } else {
                CompactTimer {}
            }
        }
    }
}
