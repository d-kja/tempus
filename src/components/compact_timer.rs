use crate::bridge;
use crate::components::timer_display::TimerDisplay;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;
use gloo_timers::callback::Interval;

#[component]
pub fn CompactTimer() -> Element {
    let state = use_context::<AppState>();
    let mut elapsed = use_signal(|| 0u64);
    let mut interval_handle = use_signal(|| Option::<Interval>::None);
    let mut poll_handle = use_signal(|| Option::<Interval>::None);
    let is_running = use_memo(move || matches!(*state.timer.read(), TimerState::Running(_)));
    let is_paused = use_memo(move || matches!(*state.timer.read(), TimerState::Stopped(_)));

    use_effect(move || {
        if *is_running.read() {
            if interval_handle.read().is_none() {
                let mut elapsed_copy = elapsed;
                let interval = Interval::new(1000, move || {
                    let current = *elapsed_copy.read();
                    elapsed_copy.set(current + 1);
                });
                *interval_handle.write() = Some(interval);
            }
        } else if matches!(*state.timer.read(), TimerState::Idle) {
            elapsed.set(0);
            *interval_handle.write() = None;
        }
    });

    use_effect(move || {
        if poll_handle.read().is_none() {
            let timer = state.timer;
            let interval = Interval::new(2000, move || {
                if matches!(*timer.read(), TimerState::Idle) {
                    let mut current_timer = timer;
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(Some(entry)) = bridge::get_active_entry().await {
                            current_timer.set(TimerState::Running(entry));
                        }
                    });
                }
            });
            *poll_handle.write() = Some(interval);
        }
    });

    let on_start = move |_| {
        spawn(async move {
            let _ = bridge::open_new_entry().await;
        });
    };

    let on_pause = {
        let mut state = state.clone();
        move |_| {
            spawn(async move {
                if let Ok(Some(entry)) = bridge::stop_entry().await {
                    state.timer.set(TimerState::Stopped(entry));
                }
            });
        }
    };

    let on_stop = {
        let mut state = state.clone();
        move |_| {
            spawn(async move {
                let _ = bridge::stop_entry().await;
                state.timer.set(TimerState::Idle);
            });
        }
    };

    let on_resume = {
        let mut state = state.clone();
        move |_| {
            if let TimerState::Stopped(entry) = state.timer.read().clone() {
                spawn(async move {
                    if let Ok(entry) = bridge::resume_entry(entry.id).await {
                        state.timer.set(TimerState::Running(entry));
                    }
                });
            }
        }
    };

    rsx! {
        div { class: "compact",
            div { class: "compact-surface",
                if *is_running.read() {
                    div { id: "timer-pill", class: "timer-pill timer-pill-running",
                        span { class: "timer-handle", "·" }
                        span { class: "timer-clock", "◷" }
                        TimerDisplay { elapsed_seconds: elapsed }
                        span { class: "timer-separator" }
                        button {
                            class: "timer-control timer-control-pause",
                            onclick: on_pause,
                            aria_label: "Pause timer",
                            "Ⅱ"
                        }
                        button {
                            class: "timer-control timer-control-stop",
                            onclick: on_stop,
                            aria_label: "Stop timer",
                            "■"
                        }
                    }
                } else if *is_paused.read() {
                    div { id: "timer-pill", class: "timer-pill timer-pill-mini",
                        TimerDisplay { elapsed_seconds: elapsed }
                        button {
                            class: "timer-state-action",
                            onclick: on_resume,
                            aria_label: "Resume timer",
                            span { class: "timer-play-icon", "▶" }
                            "Resume"
                        }
                    }
                } else {
                    div { id: "timer-pill", class: "timer-pill timer-pill-mini",
                        TimerDisplay { elapsed_seconds: elapsed }
                        button {
                            class: "timer-state-action",
                            onclick: on_start,
                            aria_label: "Start timer",
                            span { class: "timer-play-icon", "▶" }
                            "Start"
                        }
                    }
                }
            }
        }
    }
}
