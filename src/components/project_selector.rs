use crate::bridge::Project;
use dioxus::prelude::*;

#[component]
pub fn ProjectSelector(
    projects: Vec<Project>,
    selected_id: Option<i64>,
    on_select: EventHandler<Option<i64>>,
) -> Element {
    let base = "px-2.5 py-1 rounded-md text-xs font-medium transition-all active:translate-y-px";
    rsx! {
        div { class: "flex flex-wrap gap-1.5",
            button {
                class: if selected_id.is_none() {
                    "{base} bg-zinc-100 text-zinc-950"
                } else {
                    "{base} bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
                },
                onclick: move |_| on_select.call(None),
                "None"
            }
            for project in &projects {
                button {
                    class: if Some(project.id) == selected_id {
                        "{base} bg-zinc-100 text-zinc-950"
                    } else {
                        "{base} bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
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