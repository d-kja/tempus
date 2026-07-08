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
        div { class: "flex flex-col gap-4 p-4 overflow-y-auto h-full",
            if !status.read().is_empty() {
                div { class: "text-xs text-zinc-500 border border-zinc-200 rounded-md p-2", "{status}" }
            }

            section {
                h3 { class: "text-sm font-medium text-zinc-900 mb-2", "Projects" }
                div { class: "flex gap-2 mb-2",
                    input {
                        class: "flex-1 px-2 py-1 rounded-md text-sm border border-zinc-200 bg-zinc-50 placeholder-zinc-400 focus:outline-none focus:border-zinc-400",
                        placeholder: "Project name",
                        value: "{project_name}",
                        oninput: move |e| project_name.set(e.value())
                    }
                    button {
                        class: "px-3 py-1 rounded-md text-sm bg-zinc-800 text-zinc-50 hover:bg-zinc-700",
                        onclick: on_add_project,
                        "Add"
                    }
                }
                div { class: "flex flex-col gap-1",
                    for project in state.projects.read().iter() {
                        div {
                            class: "flex items-center justify-between px-2 py-1 rounded-md border border-zinc-200 text-xs",
                            span { "{project.name}" }
                            button {
                                class: "text-red-600 hover:bg-red-50 px-2 py-0.5 rounded",
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
                                "Del"
                            }
                        }
                    }
                    if state.projects.read().is_empty() {
                        p { class: "text-xs text-zinc-400", "No projects yet." }
                    }
                }
            }

            section {
                h3 { class: "text-sm font-medium text-zinc-900 mb-2", "Export" }
                div { class: "flex gap-2 mb-2",
                    input {
                        class: "flex-1 px-2 py-1 rounded-md text-sm border border-zinc-200 bg-zinc-50 placeholder-zinc-400 focus:outline-none focus:border-zinc-400",
                        placeholder: "Path e.g. /home/user/report.md",
                        value: "{export_path}",
                        oninput: move |e| export_path.set(e.value())
                    }
                    button {
                        class: "px-3 py-1 rounded-md text-sm bg-zinc-200 text-zinc-700 hover:bg-zinc-300",
                        onclick: on_save_path,
                        "Save"
                    }
                }
                button {
                    class: "w-full py-2 rounded-md text-sm bg-zinc-800 text-zinc-50 hover:bg-zinc-700",
                    onclick: on_export,
                    "Export to Markdown"
                }
            }
        }
    }
}
