use crate::bridge::{self, Entry};
use dioxus::prelude::*;

fn edit_entry_id() -> Option<i64> {
    web_sys::window()?
        .location()
        .search()
        .ok()?
        .trim_start_matches('?')
        .split('&')
        .find_map(|part| part.strip_prefix("edit-entry=")?.parse().ok())
}

fn date_part(value: &str) -> String {
    value.get(..10).unwrap_or_default().to_string()
}

fn time_part(value: &str) -> String {
    value.get(11..16).unwrap_or_default().to_string()
}

fn combine_datetime(date: &str, time: &str) -> String {
    if date.is_empty() {
        time.to_string()
    } else if time.is_empty() {
        date.to_string()
    } else {
        format!("{} {}", date, time)
    }
}

fn database_datetime(value: &str) -> Option<String> {
    let value = value.trim().replace('T', " ");
    if value.is_empty() {
        return None;
    }
    let (date, time) = value.split_once(' ')?;
    if date.len() != 10
        || time.len() != 5
        || date.as_bytes().get(4) != Some(&b'-')
        || date.as_bytes().get(7) != Some(&b'-')
        || time.as_bytes().get(2) != Some(&b':')
    {
        return None;
    }
    let month = date.get(5..7)?.parse::<u32>().ok()?;
    let day = date.get(8..10)?.parse::<u32>().ok()?;
    let hour = time.get(..2)?.parse::<u32>().ok()?;
    let minute = time.get(3..)?.parse::<u32>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }
    Some(format!("{} {}:00", date, time))
}

fn duration_label(start: &str, end: &str) -> String {
    let start_ms = js_sys::Date::parse(&start.replace(' ', "T"));
    let end_ms = if end.is_empty() {
        js_sys::Date::now()
    } else {
        js_sys::Date::parse(&end.replace(' ', "T"))
    };
    if start_ms.is_nan() || end_ms.is_nan() {
        return "—".into();
    }
    let minutes = ((end_ms - start_ms) / 60_000.0).max(0.0) as i64;
    format!("{}h {}m", minutes / 60, minutes % 60)
}

#[component]
pub fn EditEntryWindow() -> Element {
    let entry_id = edit_entry_id();
    let mut entry = use_signal(|| None::<Entry>);
    let mut projects = use_signal(Vec::new);
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut start_date = use_signal(String::new);
    let mut start_clock = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut end_clock = use_signal(String::new);
    let mut selected_project = use_signal(|| None::<i64>);
    let mut project_menu_open = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(String::new);

    use_effect(move || {
        spawn(async move {
            let Some(id) = entry_id else {
                loading.set(false);
                error.set("Entry not found.".into());
                return;
            };
            let entries = bridge::get_entries(Some(50), None).await;
            let project_list = bridge::get_projects().await;
            match entries {
                Ok(entries) => match entries.into_iter().find(|candidate| candidate.id == id) {
                    Some(loaded) => {
                        title.set(loaded.title.clone());
                        description.set(loaded.description.clone().unwrap_or_default());
                        start_date.set(date_part(&loaded.start_time));
                        start_clock.set(time_part(&loaded.start_time));
                        if let Some(end) = loaded.end_time.as_deref() {
                            end_date.set(date_part(end));
                            end_clock.set(time_part(end));
                        }
                        selected_project.set(loaded.project_id);
                        entry.set(Some(loaded));
                    }
                    None => error.set("Entry not found.".into()),
                },
                Err(message) => error.set(message),
            }
            if let Ok(project_list) = project_list {
                projects.set(project_list);
            }
            loading.set(false);
        });
    });

    let selected_id = *selected_project.read();
    let selected_project_name = selected_id
        .and_then(|id| {
            projects
                .read()
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.name.clone())
        })
        .unwrap_or_else(|| "None".into());
    let duration = use_memo(move || {
        let start = combine_datetime(&start_date.read(), &start_clock.read());
        let end = combine_datetime(&end_date.read(), &end_clock.read());
        duration_label(&start, &end)
    });

    let on_save = move |_| {
        let Some(id) = entry_id else {
            return;
        };
        let title_value = title.read().trim().to_string();
        if title_value.is_empty() {
            error.set("Title is required.".into());
            return;
        }
        let description_value = description.read().trim().to_string();
        let start_raw = combine_datetime(&start_date.read(), &start_clock.read());
        let start_value = database_datetime(&start_raw);
        let Some(start_value) = start_value else {
            error.set("Use YYYY-MM-DD and HH:MM for the start time.".into());
            return;
        };
        let end_raw = combine_datetime(&end_date.read(), &end_clock.read());
        let end_value = database_datetime(&end_raw);
        if !end_raw.trim().is_empty() && end_value.is_none() {
            error.set("Use YYYY-MM-DD and HH:MM for the end time.".into());
            return;
        }
        let project_id = *selected_project.read();
        spawn(async move {
            match bridge::update_entry(
                id,
                title_value,
                description_value,
                project_id,
                start_value,
                end_value,
            )
            .await
            {
                Ok(_) => {
                    let _ = bridge::close_current_window().await;
                }
                Err(message) => error.set(message),
            }
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        div { class: "new-entry edit-entry",
            if *loading.read() {
                div { class: "edit-entry-state", "Loading entry…" }
            } else if !error.read().is_empty() && entry.read().is_none() {
                div { class: "edit-entry-state edit-entry-error", "{error}" }
                div { class: "new-entry-actions",
                    button {
                        class: "btn btn-outline",
                        onclick: move |_| {
                            spawn(async { let _ = bridge::close_current_window().await; });
                        },
                        "Close"
                    }
                }
            } else {
                div { class: "new-entry-header",
                    div { class: "new-entry-header-copy",
                        h3 { class: "new-entry-label", "Edit tracked work" }
                        p { class: "new-entry-subtitle", "Keep the task details and time range accurate." }
                    }
                    div { class: "edit-duration-chip",
                        span { class: "edit-duration-icon", "◷" }
                        span { class: "mono", "{duration}" }
                    }
                }
                div { class: "edit-fields",
                    div { class: "edit-row",
                        label { class: "new-entry-field",
                            span { class: "field-label", "Title" }
                            input {
                                class: "field-control",
                                value: "{title}",
                                autofocus: true,
                                oninput: move |event| title.set(event.value())
                            }
                        }
                        div { class: "new-entry-field",
                            span { class: "field-label", "Project" }
                            div { class: "project-picker",
                                button {
                                    r#type: "button",
                                    class: "project-control field-control",
                                    aria_label: "Choose a project",
                                    aria_expanded: "{project_menu_open}",
                                    onclick: move |_| {
                                        let open = *project_menu_open.read();
                                        project_menu_open.set(!open);
                                    },
                                    span { class: "field-value", "{selected_project_name}" }
                                    span { class: "project-chevron", "⌄" }
                                }
                                if *project_menu_open.read() {
                                    div { class: "project-menu", role: "listbox",
                                        button {
                                            r#type: "button",
                                            class: if selected_id.is_none() { "project-option project-option-active" } else { "project-option" },
                                            onclick: move |_| { selected_project.set(None); project_menu_open.set(false); },
                                            "None"
                                        }
                                        for project in projects.read().iter() {
                                            button {
                                                r#type: "button",
                                                class: if selected_id == Some(project.id) { "project-option project-option-active" } else { "project-option" },
                                                onclick: { let id = project.id; move |_| { selected_project.set(Some(id)); project_menu_open.set(false); } },
                                                "{project.name}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "edit-row",
                        label { class: "new-entry-field",
                            span { class: "field-label", "Start time" }
                            div { class: "field-control edit-time-control",
                                span { class: "edit-time-icon", "◷" }
                                div { class: "edit-time-values",
                                    input {
                                        class: "edit-time-value",
                                        value: "{start_clock}",
                                        placeholder: "HH:MM",
                                        maxlength: "5",
                                        inputmode: "numeric",
                                        oninput: move |event| {
                                            start_clock.set(event.value());
                                        }
                                    }
                                    input {
                                        class: "edit-time-date",
                                        value: "{start_date}",
                                        placeholder: "YYYY-MM-DD",
                                        maxlength: "10",
                                        inputmode: "numeric",
                                        oninput: move |event| {
                                            start_date.set(event.value());
                                        }
                                    }
                                }
                            }
                        }
                        label { class: "new-entry-field",
                            span { class: "field-label", "End time" }
                            div { class: "field-control edit-time-control",
                                span { class: "edit-time-icon", "◷" }
                                div { class: "edit-time-values",
                                    input {
                                        class: "edit-time-value",
                                        value: "{end_clock}",
                                        placeholder: "HH:MM",
                                        maxlength: "5",
                                        inputmode: "numeric",
                                        oninput: move |event| {
                                            end_clock.set(event.value());
                                        }
                                    }
                                    input {
                                        class: "edit-time-date",
                                        value: "{end_date}",
                                        placeholder: "YYYY-MM-DD",
                                        maxlength: "10",
                                        inputmode: "numeric",
                                        oninput: move |event| {
                                            end_date.set(event.value());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    label { class: "new-entry-field",
                        span { class: "field-label", "Description" }
                        textarea {
                            class: "field-control description-control",
                            value: "{description}",
                            oninput: move |event| description.set(event.value())
                        }
                    }
                }
                if !error.read().is_empty() {
                    p { class: "edit-entry-inline-error", "{error}" }
                }
                div { class: "new-entry-actions",
                    button {
                        class: "btn btn-outline",
                        onclick: move |_| {
                            spawn(async { let _ = bridge::close_current_window().await; });
                        },
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: on_save,
                        span { class: "button-icon", "✓" }
                        "Save changes"
                    }
                }
            }
        }
    }
}
