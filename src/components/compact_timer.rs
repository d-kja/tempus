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

    let primary_label = use_memo(move || {
        if matches!(*state.timer.read(), TimerState::Idle) {
            "Start"
        } else if *is_running.read() {
            "Stop"
        } else {
            "Resume"
        }
    });

    let primary_class = use_memo(move || {
        if matches!(*state.timer.read(), TimerState::Stopped(_)) {
            "btn btn-outline"
        } else {
            "btn btn-primary"
        }
    });

    let title_text = use_memo(move || {
        let t = title.read();
        if t.is_empty() { "no active entry".to_string() } else { t.clone() }
    });

    rsx! {
        div { class: "compact",
            div { class: "compact-header",
                div { class: "compact-status",
                    span { class: if *is_running.read() { "dot dot-on" } else { "dot dot-off" } }
                    span { class: "compact-title", "{title_text}" }
                }
                button {
                    class: "icon-btn",
                    onclick: move |e: dioxus::events::MouseEvent| {
                        e.stop_propagation();
                        spawn(async move {
                            let _ = bridge::open_settings().await;
                        });
                    },
                    "\u{22EF}"
                }
            }

            div { class: "compact-timer",
                div { class: "compact-timer-inner",
                    TimerDisplay { elapsed_seconds: elapsed }
                    p { class: "label",
                        if *is_running.read() { "running" } else { "idle" }
                    }
                }
            }

            div { class: "compact-action",
                button {
                    class: "{primary_class}",
                    onclick: on_start_stop,
                    "{primary_label}"
                }
            }
        }
    }
}