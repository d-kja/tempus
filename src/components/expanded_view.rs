use crate::bridge;
use crate::components::entry_row::EntryRow;
use crate::components::navigation::{Navigation, Page};
use crate::components::project_selector::ProjectSelector;
use crate::components::setup_page::SetupPage;
use crate::components::timer_display::TimerDisplay;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;
use gloo_timers::callback::Interval;

#[component]
pub fn ExpandedView() -> Element {
    let mut state = use_context::<AppState>();
    let mut page = use_signal(|| Page::Timer);
    let mut title = use_signal(String::new);
    let mut selected_project = use_signal(|| None::<i64>);
    let mut elapsed = use_signal(|| 0u64);

    use_effect({
        let mut entries = state.entries;
        let mut projects = state.projects;
        let mut settings = state.settings;
        move || {
            wasm_bindgen_futures::spawn_local(async move {
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

    let on_start_stop = {
        let t = title.read().clone();
        let pid = *selected_project.read();
        let mut state = state.clone();
        move |_| {
            let t = t.clone();
            let mut state = state.clone();
            spawn(async move {
                let timer = state.timer.read().clone();
                match timer {
                    TimerState::Running(_) => {
                        if let Ok(Some(entry)) = bridge::stop_entry().await {
                            state.timer.set(TimerState::Stopped(entry));
                            if let Ok(entries) = bridge::get_entries(Some(20), None).await {
                                state.entries.set(entries);
                            }
                        }
                    }
                    TimerState::Stopped(_) | TimerState::Idle => {
                        if !t.is_empty() {
                            if let Ok(entry) = bridge::start_entry(t, None, pid).await {
                                state.timer.set(TimerState::Running(entry));
                            }
                        }
                    }
                }
            });
        }
    };

    let on_minimize = {
        let mut state = state.clone();
        move |_| {
            state.is_expanded.set(false);
            spawn(async move {
                let _ = bridge::set_window_size(320.0, 160.0).await;
            });
        }
    };

    let primary_label = use_memo(move || {
        if *is_running.read() { "Stop" } else if matches!(*state.timer.read(), TimerState::Stopped(_)) { "Resume" } else { "Start" }
    });

    let primary_class = use_memo(move || {
        if *is_running.read() {
            "btn btn-primary"
        } else if matches!(*state.timer.read(), TimerState::Stopped(_)) {
            "btn btn-outline"
        } else {
            "btn btn-primary"
        }
    });

    let subtitle = use_memo(move || {
        match &*state.timer.read() {
            TimerState::Running(e) | TimerState::Stopped(e) => Some(e.title.clone()),
            TimerState::Idle => None,
        }
    });

    let timer_text = use_memo(move || {
        let total = *elapsed.read();
        format!("{:02}:{:02}:{:02}", total / 3600, (total % 3600) / 60, total % 60)
    });

    rsx! {
        div { class: "expanded",
            div { class: "expanded-header",
                div { class: "expanded-header-left",
                    if *is_running.read() {
                        span { class: "dot dot-on" }
                    }
                    h2 { class: "expanded-title", "Hours" }
                }
                button {
                    class: "collapse-btn",
                    onclick: on_minimize,
                    "\u{2193} Collapse"
                }
            }

            div { class: "expanded-body",
                if *page.read() == Page::Timer {
                    div { class: "timer-page",
                        div { class: "timer-block",
                            div { class: "mono timer-md", "{timer_text}" }
                            if let Some(t) = &*subtitle.read() {
                                span { class: "timer-subtitle", "{t}" }
                            } else {
                                span { class: "timer-subtitle timer-subtitle-dim", "no active entry" }
                            }
                        }

                        div { class: "form-block",
                            input {
                                class: "input",
                                placeholder: "What are you working on?",
                                value: "{title}",
                                oninput: move |e| title.set(e.value())
                            }
                            ProjectSelector {
                                projects: state.projects.read().clone(),
                                selected_id: *selected_project.read(),
                                on_select: move |id| selected_project.set(id),
                            }
                            button {
                                class: "{primary_class}",
                                onclick: on_start_stop,
                                "{primary_label}"
                            }
                        }

                        div { class: "entries",
                            h3 { class: "entries-header", "Recent" }
                            if state.entries.read().is_empty() {
                                p { class: "entries-empty", "No entries yet." }
                            }
                            for entry in state.entries.read().iter() {
                                EntryRow { entry: entry.clone() }
                            }
                        }
                    }
                } else if *page.read() == Page::Setup {
                    SetupPage {}
                }
            }

            Navigation { current: page }
        }
    }
}