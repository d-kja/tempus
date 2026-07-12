use crate::bridge;
use dioxus::prelude::*;

#[component]
pub fn NewEntryWindow() -> Element {
    let mut title = use_signal(String::new);
    let mut description = use_signal(String::new);
    let mut projects = use_signal(Vec::new);
    let mut selected_project = use_signal(|| None::<i64>);
    let mut project_menu_open = use_signal(|| false);

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
                div { class: "new-entry-field",
                    span { class: "field-label", "Project" }
                    div { class: "project-picker",
                        button {
                            r#type: "button",
                            class: "project-control field-control",
                            aria_label: "Choose a project",
                            aria_expanded: "{project_menu_open}",
                            onclick: move |_| {
                                let open = { *project_menu_open.read() };
                                project_menu_open.set(!open);
                            },
                            span {
                                class: if selected_id.is_some() { "field-value" } else { "field-value field-value-muted" },
                                "{selected_project_name}"
                            }
                            span { class: "field-hint", "Choose a project" }
                            span { class: "project-chevron", "⌄" }
                        }
                        if *project_menu_open.read() {
                            div { class: "project-menu", role: "listbox",
                                button {
                                    r#type: "button",
                                    class: if selected_id.is_none() { "project-option project-option-active" } else { "project-option" },
                                    role: "option",
                                    aria_selected: "{selected_id.is_none()}",
                                    onclick: move |_| {
                                        selected_project.set(None);
                                        project_menu_open.set(false);
                                    },
                                    span { "None" }
                                }
                                for p in projects.read().iter() {
                                    button {
                                        r#type: "button",
                                        class: if selected_id == Some(p.id) { "project-option project-option-active" } else { "project-option" },
                                        role: "option",
                                        aria_selected: "{selected_id == Some(p.id)}",
                                        onclick: { let pid = p.id; move |_| {
                                            selected_project.set(Some(pid));
                                            project_menu_open.set(false);
                                        }},
                                        span { "{p.name}" }
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
