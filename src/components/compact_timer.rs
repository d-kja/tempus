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
            "w-full py-2 rounded-md text-sm font-medium active:translate-y-px transition-all \
             border border-zinc-700 text-zinc-200 hover:bg-zinc-800"
        } else {
            "w-full py-2 rounded-md text-sm font-medium active:translate-y-px transition-all \
             bg-zinc-100 text-zinc-950 hover:bg-white"
        }
    });

    rsx! {
        div {
            class: "h-full w-full flex flex-col select-none",
            ondblclick: on_expand,

            // Header: status + expand
            div {
                class: "flex items-center justify-between px-3 pt-2.5",
                div { class: "flex items-center gap-1.5 min-w-0",
                    if *is_running.read() {
                        span { class: "w-1.5 h-1.5 rounded-full bg-emerald-400 shrink-0" }
                    } else {
                        span { class: "w-1.5 h-1.5 rounded-full bg-zinc-600 shrink-0" }
                    }
                    span {
                        class: "text-[11px] text-zinc-400 truncate",
                        if title.read().is_empty() { "no active entry" } else { "{title}" }
                    }
                }
                button {
                    class: "text-zinc-500 hover:text-zinc-100 active:translate-y-px transition-colors -mr-0.5",
                    onclick: {
                        let mut state = state.clone();
                        move |e: dioxus::events::MouseEvent| {
                            e.stop_propagation();
                            state.is_expanded.set(true);
                            spawn(async move {
                                let _ = bridge::set_window_size(340.0, 480.0).await;
                            });
                        }
                    },
                    "\u{22EF}"
                }
            }

            // Timer
            div {
                class: "flex-1 flex items-center justify-center",
                div {
                    class: "text-center",
                    div { class: "text-4xl",
                        TimerDisplay { elapsed_seconds: elapsed }
                    }
                    p {
                        class: "text-[10px] uppercase tracking-[0.18em] text-zinc-500 mt-1.5",
                        if *is_running.read() { "running" } else { "idle" }
                    }
                }
            }

            // Action
            div { class: "px-3 pb-3",
                button {
                    class: "{primary_class}",
                    onclick: on_start_stop,
                    "{primary_label}"
                }
            }
        }
    }
}