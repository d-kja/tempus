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
        div { class: "flex flex-col gap-6 p-4 overflow-y-auto h-full",
            if !status.read().is_empty() {
                div { class: "text-xs text-zinc-700 bg-zinc-100 border border-zinc-200 rounded-md px-3 py-2",
                    "{status}"
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-[10px] font-medium uppercase tracking-[0.18em] text-zinc-400", "Projects" }
                div { class: "flex gap-1.5",
                    input {
                        class: "flex-1 px-3 py-1.5 rounded-md text-sm bg-white border border-zinc-200 \
                                placeholder-zinc-400 text-zinc-900 \
                                focus:outline-none focus:border-zinc-400 focus:ring-1 focus:ring-zinc-200",
                        placeholder: "New project name",
                        value: "{project_name}",
                        oninput: move |e| project_name.set(e.value())
                    }
                    button {
                        class: "px-3 py-1.5 rounded-md text-sm font-medium \
                                bg-zinc-900 text-zinc-50 hover:bg-zinc-800 active:translate-y-px transition-all",
                        onclick: on_add_project,
                        "Add"
                    }
                }
                div { class: "divide-y divide-zinc-100",
                    for project in state.projects.read().iter() {
                        div {
                            class: "flex items-center justify-between py-2",
                            span { class: "text-sm text-zinc-800 truncate", "{project.name}" }
                            button {
                                class: "shrink-0 text-xs text-zinc-400 hover:text-red-600 active:translate-y-px transition-colors px-1",
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
                        p { class: "py-3 text-xs text-zinc-400", "No projects yet." }
                    }
                }
            }

            section { class: "flex flex-col gap-2",
                h3 { class: "text-[10px] font-medium uppercase tracking-[0.18em] text-zinc-400", "Export Path" }
                div { class: "flex gap-1.5",
                    input {
                        class: "flex-1 px-3 py-1.5 rounded-md text-xs font-mono bg-white border border-zinc-200 \
                                placeholder-zinc-400 text-zinc-900 \
                                focus:outline-none focus:border-zinc-400 focus:ring-1 focus:ring-zinc-200",
                        placeholder: "/home/user/report.md",
                        value: "{export_path}",
                        oninput: move |e| export_path.set(e.value())
                    }
                    button {
                        class: "px-3 py-1.5 rounded-md text-sm font-medium \
                                bg-zinc-100 text-zinc-700 hover:bg-zinc-200 active:translate-y-px transition-all",
                        onclick: on_save_path,
                        "Save"
                    }
                }
                button {
                    class: "w-full py-2 rounded-md text-sm font-medium \
                            bg-zinc-900 text-zinc-50 hover:bg-zinc-800 active:translate-y-px transition-all",
                    onclick: on_export,
                    "Export Markdown"
                }
            }
        }
    }
}