use crate::bridge;
use crate::bridge::Project;
use dioxus::prelude::*;

#[component]
pub fn NewEntryWindow() -> Element {
    let mut title = use_signal(|| "Untitled".to_string());
    let mut projects = use_signal(Vec::new);
    let mut selected_project = use_signal(|| None::<i64>);

    use_effect(move || {
        spawn(async move {
            if let Ok(p) = bridge::get_projects().await {
                projects.set(p);
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        div { class: "new-entry",
            h3 { class: "new-entry-label", "What are you working on?" }
            input {
                class: "input",
                value: "{title}",
                autofocus: true,
                oninput: move |e| title.set(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        let t = title.read().clone();
                        let pid = *selected_project.read();
                        spawn(async move {
                            let _ = bridge::start_entry(t, None, pid).await;
                            let _ = bridge::close_current_window().await;
                        });
                    }
                }
            }
            div { class: "new-entry-section",
                h4 { class: "section-label-sm", "Project (optional)" }
                div { class: "project-chips",
                    button {
                        class: if selected_project.read().is_none() { "chip chip-active" } else { "chip" },
                        onclick: move |_| selected_project.set(None),
                        "None"
                    }
                    for p in projects.read().iter() {
                        button {
                            class: if *selected_project.read() == Some(p.id) { "chip chip-active" } else { "chip" },
                            onclick: { let pid = p.id; move |_| selected_project.set(Some(pid)) },
                            "{p.name}"
                        }
                    }
                }
            }
            div { class: "spacer" }
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
                        let pid = *selected_project.read();
                        spawn(async move {
                            let _ = bridge::start_entry(t, None, pid).await;
                            let _ = bridge::close_current_window().await;
                        });
                    },
                    "Start"
                }
            }
        }
    }
}