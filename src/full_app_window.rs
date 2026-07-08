use crate::bridge;
use crate::components::entry_row::EntryRow;
use crate::components::navigation::{Navigation, Page};
use crate::components::project_selector::ProjectSelector;
use crate::state::TimerState;
use dioxus::prelude::*;
use gloo_timers::callback::Interval;
use std::collections::HashMap;

static CSS: Asset = asset!("/assets/app.css");

#[component]
pub fn FullAppWindow() -> Element {
    let page = use_signal(|| Page::Timer);
    let mut elapsed = use_signal(|| 0u64);
    let mut entries_sig = use_signal(Vec::new);
    let mut projects_sig = use_signal(Vec::new);
    let mut settings_sig = use_signal(HashMap::new);
    let mut timer_state = use_signal(|| TimerState::Idle);
    let mut project_name = use_signal(String::new);
    let mut export_path = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut filter_project = use_signal(|| None::<i64>);
    let mut interval_handle = use_signal(|| Option::<Interval>::None);
    let mut clear_confirm = use_signal(|| false);
    let is_running = use_memo(move || matches!(*timer_state.read(), TimerState::Running(_)));

    use_effect(move || {
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(active) = bridge::get_active_entry().await {
                if let Some(entry) = active {
                    timer_state.set(TimerState::Running(entry));
                }
            }
            if let Ok(e) = bridge::get_entries(Some(50), None).await {
                entries_sig.set(e);
            }
            if let Ok(p) = bridge::get_projects().await {
                projects_sig.set(p);
            }
            if let Ok(s) = bridge::get_settings().await {
                let path = s.get("export_path").cloned().unwrap_or_default();
                export_path.set(path);
                settings_sig.set(s);
            }
        });
    });

    use_effect(move || {
        let is_running = *is_running.read();
        if is_running {
            if interval_handle.read().is_none() {
                let mut e = elapsed;
                let interval = Interval::new(1000, move || {
                    let current = *e.read();
                    e.set(current + 1);
                });
                *interval_handle.write() = Some(interval);
            }
        } else {
            elapsed.set(0);
            *interval_handle.write() = None;
        }
    });

    let on_delete_entry = move |id: i64| {
        spawn(async move {
            let _ = bridge::delete_entry(id).await;
            if let Ok(e) = bridge::get_entries(Some(50), None).await {
                entries_sig.set(e);
            }
        });
    };

    let on_close = move |_| {
        spawn(async move {
            let _ = bridge::close_current_window().await;
        });
    };

    let always_on_top = use_memo(move || {
        settings_sig.read().get("always_on_top").cloned().unwrap_or("true".into()) == "true"
    });

    let on_toggle_aot = move |_| {
        let new = !*always_on_top.read();
        let mut s = settings_sig.read().clone();
        s.insert("always_on_top".into(), new.to_string());
        settings_sig.set(s.clone());
        spawn(async move {
            let _ = bridge::set_always_on_top(new).await;
            let _ = bridge::update_settings(s).await;
        });
    };

    let on_add_project = move |_| {
        let name = project_name.read().clone();
        if name.is_empty() { return; }
        spawn(async move {
            match bridge::create_project(name).await {
                Ok(_) => {
                    project_name.set(String::new());
                    if let Ok(projects) = bridge::get_projects().await {
                        projects_sig.set(projects);
                    }
                    status.set("Project added.".into());
                }
                Err(e) => status.set(format!("Error: {}", e)),
            }
        });
    };

    let on_export = move |_| {
        let path = export_path.read().clone();
        spawn(async move {
            match bridge::export_markdown(path).await {
                Ok(_) => status.set("Exported successfully.".into()),
                Err(e) => status.set(format!("Export error: {}", e)),
            }
        });
    };

    let on_save_path = move |_| {
        let path = export_path.read().clone();
        let mut s = settings_sig.read().clone();
        s.insert("export_path".into(), path);
        spawn(async move {
            let _ = bridge::update_settings(s).await;
            status.set("Path saved.".into());
        });
    };

    let timer_text = use_memo(move || {
        let total = *elapsed.read();
        format!("{:02}:{:02}:{:02}", total / 3600, (total % 3600) / 60, total % 60)
    });

    let running_title = use_memo(move || {
        match &*timer_state.read() {
            TimerState::Running(e) => e.title.clone(),
            _ => String::new(),
        }
    });

    let filtered_entries = use_memo(move || {
        let all = entries_sig.read();
        let pid = *filter_project.read();
        match pid {
            Some(id) => all.iter().filter(|e| e.project_id == Some(id)).cloned().collect::<Vec<_>>(),
            None => all.clone(),
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "expanded",
            div { class: "expanded-header",
                div { class: "expanded-header-left",
                    if *is_running.read() {
                        span { class: "dot dot-on" }
                    }
                    h2 { class: "expanded-title", "Hours" }
                }
                div { class: "expanded-header-right",
                    button {
                        class: "collapse-btn",
                        onclick: on_close,
                        "\u{00D7}"
                    }
                }
            }

            if !status.read().is_empty() {
                div { class: "status-box", "{status}" }
            }

            if *page.read() == Page::Timer {
                div { class: "expanded-body",
                    if *is_running.read() {
                        div { class: "running-banner",
                            span { class: "running-dot" }
                            span { class: "running-text", "{running_title}" }
                            span { class: "mono running-time", "{timer_text}" }
                        }
                    }
                    div { class: "entries-filter",
                        ProjectSelector {
                            projects: projects_sig.read().clone(),
                            selected_id: *filter_project.read(),
                            on_select: move |id| filter_project.set(id),
                        }
                    }
                    div { class: "entries",
                        div { class: "entries-header-row",
                            h3 { class: "entries-header", "Recent entries" }
                            button {
                                class: "btn-icon btn-icon-danger",
                                onclick: move |_| clear_confirm.set(true),
                                "\u{1F5D1}"
                            }
                        }
                        if *clear_confirm.read() {
                            div { class: "confirm-clear",
                                span { "Clear all entries?" }
                                div { class: "confirm-clear-actions",
                                    button {
                                        class: "btn btn-sm btn-danger",
                                        onclick: move |_| {
                                            spawn(async move {
                                                let _ = bridge::clear_all_entries().await;
                                                entries_sig.set(Vec::new());
                                                clear_confirm.set(false);
                                            });
                                        },
                                        "Yes, clear"
                                    }
                                    button {
                                        class: "btn btn-sm btn-outline",
                                        onclick: move |_| clear_confirm.set(false),
                                        "Cancel"
                                    }
                                }
                            }
                        }
                        if filtered_entries.read().is_empty() {
                            p { class: "entries-empty", "No entries yet." }
                        }
                        for entry in filtered_entries.read().iter() {
                            EntryRow {
                                entry: entry.clone(),
                                on_delete: on_delete_entry,
                            }
                        }
                    }
                }
            } else if *page.read() == Page::Setup {
                div { class: "expanded-body",
                    div { class: "page",
                        section { class: "section",
                            h3 { class: "section-label", "Projects" }
                            div { class: "row",
                                input {
                                    class: "input",
                                    placeholder: "New project name",
                                    value: "{project_name}",
                                    oninput: move |e| project_name.set(e.value())
                                }
                                button {
                                    class: "btn btn-sm btn-primary",
                                    onclick: on_add_project,
                                    "Add"
                                }
                            }
                            div { class: "project-list",
                                for project in projects_sig.read().iter() {
                                    div { class: "project-item",
                                        span { class: "project-name", "{project.name}" }
                                        button {
                                            class: "delete-btn",
                                            onclick: {
                                                let pid = project.id;
                                                move |_| {
                                                    spawn(async move {
                                                        let _ = bridge::delete_project(pid).await;
                                                        if let Ok(projects) = bridge::get_projects().await {
                                                            projects_sig.set(projects);
                                                        }
                                                    });
                                                }
                                            },
                                            "Delete"
                                        }
                                    }
                                }
                                if projects_sig.read().is_empty() {
                                    p { class: "entries-empty", "No projects yet." }
                                }
                            }
                        }

                        section { class: "section",
                            h3 { class: "section-label", "Export Path" }
                            div { class: "row",
                                input {
                                    class: "input input-mono",
                                    placeholder: "/home/user/report.md",
                                    value: "{export_path}",
                                    oninput: move |e| export_path.set(e.value())
                                }
                                button {
                                    class: "btn btn-sm btn-outline",
                                    onclick: on_save_path,
                                    "Save"
                                }
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: on_export,
                                "Export Markdown"
                            }
                        }
                        div { class: "page-filler" }
                    }
                }
            } else {
                div { class: "expanded-body",
                    div { class: "page",
                        section { class: "section",
                            h3 { class: "section-label", "Display" }
                            button {
                                class: "toggle-row",
                                onclick: on_toggle_aot,
                                span { class: "toggle-label", "Always on top" }
                                span {
                                    class: if *always_on_top.read() { "toggle toggle-on" } else { "toggle toggle-off" },
                                    span {
                                        class: if *always_on_top.read() { "toggle-knob toggle-knob-on" } else { "toggle-knob toggle-knob-off" }
                                    }
                                }
                            }
                            p { class: "helper-text", "Keep the timer visible above all other windows." }
                        }
                        div { class: "page-filler" }
                    }
                }
            }

            Navigation { current: page }
        }
    }
}
