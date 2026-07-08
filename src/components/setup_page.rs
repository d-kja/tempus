use crate::bridge;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SetupPage() -> Element {
    let mut state = use_context::<AppState>();
    let mut project_name = use_signal(String::new);
    let mut export_path = use_signal(String::new);
    let mut status = use_signal(String::new);

    use_effect({
        let state = state.clone();
        move || {
            let path = state
                .settings
                .read()
                .get("export_path")
                .cloned()
                .unwrap_or_default();
            export_path.set(path);
        }
    });

    let on_add_project = {
        let state = state.clone();
        move |_| {
            let name = project_name.read().clone();
            if name.is_empty() {
                return;
            }
            let mut state = state.clone();
            spawn(async move {
                match bridge::create_project(name).await {
                    Ok(_) => {
                        project_name.set(String::new());
                        if let Ok(projects) = bridge::get_projects().await {
                            state.projects.set(projects);
                        }
                        status.set("Project added.".into());
                    }
                    Err(e) => status.set(format!("Error: {}", e)),
                }
            });
        }
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
        let mut s = state.settings.read().clone();
        s.insert("export_path".into(), path);
        spawn(async move {
            let _ = bridge::update_settings(s).await;
            status.set("Path saved.".into());
        });
    };

    rsx! {
        div { class: "page",
            if !status.read().is_empty() {
                div { class: "status-box", "{status}" }
            }

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
                    for project in state.projects.read().iter() {
                        div { class: "project-item",
                            span { class: "project-name", "{project.name}" }
                            button {
                                class: "delete-btn",
                                onclick: {
                                    let pid = project.id;
                                    let mut state = state.clone();
                                    move |_| {
                                        let mut state = state.clone();
                                        spawn(async move {
                                            let _ = bridge::delete_project(pid).await;
                                            if let Ok(projects) = bridge::get_projects().await {
                                                state.projects.set(projects);
                                            }
                                        });
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                    if state.projects.read().is_empty() {
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
        }
    }
}