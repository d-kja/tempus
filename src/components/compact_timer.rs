use crate::bridge;
use crate::components::timer_display::TimerDisplay;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;
use gloo_timers::callback::Interval;

#[component]
pub fn CompactTimer() -> Element {
    let mut state = use_context::<AppState>();
    let mut elapsed = use_signal(|| 0u64);

    use_effect(move || {
        let timer_state = state.timer.read().clone();
        if matches!(timer_state, TimerState::Running(_)) {
            let mut elapsed_copy = elapsed;
            let interval = Interval::new(1000, move || {
                let current = *elapsed_copy.read();
                elapsed_copy.set(current + 1);
            });
            std::mem::forget(interval);
        } else {
            elapsed.set(0);
        }
    });

    let is_running = use_memo(move || matches!(*state.timer.read(), TimerState::Running(_)));
    let title = use_memo(move || {
        match &*state.timer.read() {
            TimerState::Running(e) | TimerState::Stopped(e) => e.title.clone(),
            TimerState::Idle => String::new(),
        }
    });

    let on_start_stop = {
        let mut state = state.clone();
        move |_| {
            spawn(async move {
            let timer = state.timer.read().clone();
            match timer {
                TimerState::Running(_) => {
                    if let Ok(Some(entry)) = bridge::stop_entry().await {
                        state.timer.set(TimerState::Stopped(entry));
                    }
                }
                TimerState::Stopped(entry) => {
                    let title = entry.title.clone();
                    if let Ok(new_entry) = bridge::start_entry(title, None, None).await {
                        state.timer.set(TimerState::Running(new_entry));
                    }
                }
                TimerState::Idle => {
                    if let Ok(entry) = bridge::start_entry("Untitled".into(), None, None).await {
                        state.timer.set(TimerState::Running(entry));
                    }
                }
            }
        });
    }
    };

    let on_expand = {
        let mut state = state.clone();
        move |_: dioxus::events::MouseEvent| {
            state.is_expanded.set(true);
            spawn(async move {
                let _ = bridge::set_window_size(340.0, 480.0).await;
            });
        }
    };

    let on_gear = {
        let mut state = state.clone();
        move |_| {
            state.is_expanded.set(true);
            spawn(async move {
                let _ = bridge::set_window_size(340.0, 480.0).await;
            });
        }
    };

    rsx! {
        div {
            class: "h-full w-full bg-zinc-50 flex flex-col items-center justify-center p-4 select-none",
            ondblclick: on_expand,
            div { class: "w-full text-center mb-2",
                span { class: "text-sm text-zinc-500", "{title}" }
            }
            TimerDisplay { elapsed_seconds: elapsed }
            div { class: "mt-4 flex items-center gap-3",
                button {
                    class: "px-4 py-2 rounded-md text-sm bg-zinc-800 text-zinc-50 hover:bg-zinc-700 transition-colors",
                    onclick: on_start_stop,
                    if matches!(*state.timer.read(), TimerState::Idle) { "Start" }
                    else if *is_running.read() { "Stop" }
                    else { "Resume" }
                }
                button {
                    class: "px-3 py-2 rounded-md text-sm text-zinc-500 hover:bg-zinc-200 transition-colors",
                    onclick: on_gear,
                    "\u{2699}"
                }
            }
        }
    }
}
