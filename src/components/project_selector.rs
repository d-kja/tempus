use crate::bridge::Project;
use dioxus::prelude::*;

#[component]
pub fn ProjectSelector(
    projects: Vec<Project>,
    selected_id: Option<i64>,
    on_select: EventHandler<Option<i64>>,
) -> Element {
    rsx! {
        div { class: "flex flex-wrap gap-1 p-2",
            button {
                class: if selected_id.is_none() {
                    "px-2 py-1 rounded-md text-xs bg-zinc-800 text-zinc-50"
                } else {
                    "px-2 py-1 rounded-md text-xs bg-zinc-100 text-zinc-500 hover:bg-zinc-200 transition-colors"
                },
                onclick: move |_| on_select.call(None),
                "None"
            }
            for project in &projects {
                button {
                    class: if Some(project.id) == selected_id {
                        "px-2 py-1 rounded-md text-xs bg-zinc-800 text-zinc-50"
                    } else {
                        "px-2 py-1 rounded-md text-xs bg-zinc-100 text-zinc-500 hover:bg-zinc-200 transition-colors"
                    },
                    onclick: {
                        let pid = project.id;
                        move |_| on_select.call(Some(pid))
                    },
                    "{project.name}"
                }
            }
        }
    }
}
