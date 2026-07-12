use crate::bridge;
use dioxus::prelude::*;

#[component]
pub fn NewEntryWindow() -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut projects = use_signal(Vec::new);
    let mut selected_project = use_signal(|| None::<i64>);

    use_effect(move || {
        spawn(async move {
            if let Ok(p) = bridge::get_projects().await {
                projects.set(p);
            }
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
        .unwrap_or_else(|| "None".to_string());
    let selected_project_value = selected_id.map(|id| id.to_string()).unwrap_or_default();

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        div { class: "new-entry",
            div { class: "new-entry-header",
                div { class: "new-entry-header-copy",
                    h3 { class: "new-entry-label", "What are you working on?" }
                    p { class: "new-entry-subtitle", "Set the task context once, then start tracking." }
                }
                div { class: "draft-chip",
                    span { class: "draft-dot" }
                    span { "Draft" }
                }
            }
            div { class: "new-entry-fields",
                label { class: "new-entry-field",
                    span { class: "field-label", "Title" }
                    input {
                        class: "field-control",
                        placeholder: "Type the task title",
                        value: "{title}",
                        autofocus: true,
                        oninput: move |e| title.set(e.value())
                    }
                }
                label { class: "new-entry-field",
                    span { class: "field-label", "Project" }
                    div { class: "project-control field-control",
                        span {
                            class: if selected_id.is_some() { "field-value" } else { "field-value field-value-muted" },
                            "{selected_project_name}"
                        }
                        span { class: "field-hint", "Choose a project" }
                        select {
                            aria_label: "Choose a project",
                            value: selected_project_value,
                            onchange: move |e| selected_project.set(e.value().parse::<i64>().ok()),
                            option { value: "", "None" }
                            for p in projects.read().iter() {
                                option { value: "{p.id}", "{p.name}" }
                            }
                        }
                    }
                }
                label { class: "new-entry-field",
                    span { class: "field-label", "Description" }
                    textarea {
                        class: "field-control description-control",
                        placeholder: "Summarize the scope, expected outcome, or handoff notes before the timer starts.",
                        value: "{description}",
                        oninput: move |e| description.set(e.value())
                    }
                }
            }
            div { class: "new-entry-actions",
                button {
                    class: "btn btn-outline",
                    onclick: move |_| {
                        spawn(async move {
                            let _ = bridge::close_current_window().await;
                        });
                    },
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let t = title.read().clone();
                        let d = description.read().trim().to_string();
                        let desc = (!d.is_empty()).then_some(d);
                        let pid = *selected_project.read();
                        spawn(async move {
                            let _ = bridge::start_entry(t, desc, pid).await;
                            let _ = bridge::close_current_window().await;
                        });
                    },
                    span { class: "button-icon", "✓" }
                    "Save"
                }
            }
        }
    }
}
