use crate::bridge::Project;
use dioxus::prelude::*;

#[component]
pub fn ProjectSelector(
    projects: Vec<Project>,
    selected_id: Option<i64>,
    on_select: EventHandler<Option<i64>>,
) -> Element {
    rsx! {
        div { class: "project-selector",
            button {
                class: if selected_id.is_none() { "pill pill-on" } else { "pill pill-off" },
                onclick: move |_| on_select.call(None),
                "None"
            }
            for project in &projects {
                button {
                    class: if Some(project.id) == selected_id { "pill pill-on" } else { "pill pill-off" },
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