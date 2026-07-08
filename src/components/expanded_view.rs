use crate::bridge;
use crate::components::entry_row::EntryRow;
use crate::components::navigation::{Navigation, Page};
use crate::components::project_selector::ProjectSelector;
use crate::components::setup_page::SetupPage;
use crate::components::settings_page::SettingsPage;
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
            "w-full py-2 rounded-md text-sm font-medium active:translate-y-px transition-all \
             bg-zinc-900 text-zinc-50 hover:bg-zinc-800"
        } else if matches!(*state.timer.read(), TimerState::Stopped(_)) {
            "w-full py-2 rounded-md text-sm font-medium active:translate-y-px transition-all \
             border border-zinc-300 text-zinc-700 hover:bg-zinc-100"
        } else {
            "w-full py-2 rounded-md text-sm font-medium active:translate-y-px transition-all \
             bg-zinc-900 text-zinc-50 hover:bg-zinc-800"
        }
    });

    rsx! {
        div { class: "h-full w-full bg-zinc-50 flex flex-col select-none",

            // Header
            div { class: "flex items-center justify-between px-4 py-3 border-b border-zinc-100",
                div { class: "flex items-center gap-2",
                    if *is_running.read() {
                        span { class: "w-1.5 h-1.5 rounded-full bg-emerald-500" }
                    }
                    h2 { class: "text-sm font-medium text-zinc-900", "Hours" }
                }
                button {
                    class: "text-xs text-zinc-500 hover:text-zinc-900 active:translate-y-px transition-colors",
                    onclick: on_minimize,
                    "\u{2193} Collapse"
                }
            }

            // Body
            div { class: "flex-1 overflow-hidden",
                if *page.read() == Page::Timer {
                    div { class: "flex flex-col h-full",

                        // Timer block
                        div { class: "flex flex-col items-center justify-center pt-5 pb-3",
                            TimerDisplay { elapsed_seconds: elapsed }
                            if let Some(entry) = match &*state.timer.read() {
                                TimerState::Running(e) | TimerState::Stopped(e) => Some(e),
                                TimerState::Idle => None,
                            } {
                                span { class: "text-[11px] text-zinc-500 mt-1.5", "{entry.title}" }
                            } else {
                                span { class: "text-[11px] text-zinc-400 mt-1.5", "no active entry" }
                            }
                        }

                        // Quick log form
                        div { class: "px-4 flex flex-col gap-2",
                            input {
                                class: "w-full px-3 py-2 rounded-md text-sm bg-white border border-zinc-200 \
                                        placeholder-zinc-400 text-zinc-900 \
                                        focus:outline-none focus:border-zinc-400 focus:ring-1 focus:ring-zinc-200",
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

                        // Recent entries
                        div { class: "flex-1 overflow-y-auto mt-4 border-t border-zinc-100",
                            h3 { class: "px-4 pt-3 pb-1.5 text-[10px] font-medium uppercase tracking-[0.18em] text-zinc-400",
                                "Recent"
                            }
                            if state.entries.read().is_empty() {
                                p { class: "px-4 py-6 text-xs text-zinc-400 text-center", "No entries yet." }
                            }
                            div { class: "divide-y divide-zinc-100",
                                for entry in state.entries.read().iter() {
                                    EntryRow { entry: entry.clone() }
                                }
                            }
                        }
                    }
                } else if *page.read() == Page::Setup {
                    SetupPage {}
                } else {
                    SettingsPage {}
                }
            }

            Navigation { current: page }
        }
    }
}