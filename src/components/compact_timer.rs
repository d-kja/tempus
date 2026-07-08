use crate::bridge;
use crate::components::timer_display::TimerDisplay;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;
use gloo_timers::callback::Interval;

#[component]
pub fn CompactTimer() -> Element {
    let state = use_context::<AppState>();
    let mut elapsed = use_signal(|| 0u64);
    let mut projects = use_signal(Vec::new);

    let mut interval_handle = use_signal(|| Option::<Interval>::None);
    let mut poll_handle = use_signal(|| Option::<Interval>::None);
    let is_running = use_memo(move || matches!(*state.timer.read(), TimerState::Running(_)));

    let has_entry = use_memo(move || !matches!(*state.timer.read(), TimerState::Idle));

    use_effect(move || {
        spawn(async move {
            if let Ok(p) = bridge::get_projects().await {
                projects.set(p);
            }
        });
    });

    use_effect(move || {
        let is_running = *is_running.read();
        if is_running {
            if interval_handle.read().is_none() {
                let mut elapsed_copy = elapsed;
                let interval = Interval::new(1000, move || {
                    let current = *elapsed_copy.read();
                    elapsed_copy.set(current + 1);
                });
                *interval_handle.write() = Some(interval);
            }
        } else {
            elapsed.set(0);
            *interval_handle.write() = None;
        }
    });

    use_effect(move || {
        if poll_handle.read().is_none() {
            let timer = state.timer;
            let interval = Interval::new(2000, move || {
                let current = timer.read().clone();
                if matches!(current, TimerState::Idle) {
                    let mut t = timer;
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(Some(entry)) = bridge::get_active_entry().await {
                            t.set(TimerState::Running(entry));
                        }
                    });
                }
            });
            *poll_handle.write() = Some(interval);
        }
    });

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
                        let _ = bridge::stop_entry().await;
                        state.timer.set(TimerState::Idle);
                    }
                    TimerState::Stopped(_) | TimerState::Idle => {
                        let _ = bridge::open_new_entry().await;
                    }
                }
            });
        }
    };

    let on_reset = {
        let mut state = state.clone();
        move |_| {
            spawn(async move {
                let timer = state.timer.read().clone();
                match timer {
                    TimerState::Running(entry) | TimerState::Stopped(entry) => {
                        let _ = bridge::delete_entry(entry.id).await;
                        state.timer.set(TimerState::Idle);
                    }
                    TimerState::Idle => {}
                }
            });
        }
    };

    let primary_label = use_memo(move || {
        if *is_running.read() { "Stop" } else { "Start" }
    });

    let title_text = use_memo(move || {
        let t = title.read();
        if t.is_empty() { "no active entry".to_string() } else { t.clone() }
    });

    let project_name = use_memo(move || {
        match &*state.timer.read() {
            TimerState::Running(e) | TimerState::Stopped(e) => {
                e.project_id
                    .and_then(|pid| {
                        projects
                            .read()
                            .iter()
                            .find(|p| p.id == pid)
                            .map(|p| p.name.clone())
                    })
                    .unwrap_or_default()
            }
            TimerState::Idle => String::new(),
        }
    });

    rsx! {
        div { class: "compact",
            div { class: "compact-header",
                div { class: "compact-status",
                    span { class: if *is_running.read() { "dot dot-on" } else { "dot dot-off" } }
                    span { class: "compact-title",
                        if project_name.read().is_empty() {
                            "{title_text}"
                        } else {
                            "{title_text} \u{00B7} {project_name}"
                        }
                    }
                }
                div { class: "compact-header-actions",
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
                div { class: "action-row",
                    button {
                        class: "btn btn-primary",
                        onclick: on_start_stop,
                        "{primary_label}"
                    }
                    if *has_entry.read() {
                        button {
                            class: "reset-btn",
                            onclick: on_reset,
                            "\u{21BA}"
                        }
                    }
                }
            }
        }
    }
}
