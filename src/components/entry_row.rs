use crate::bridge::{Entry, Project};
use dioxus::prelude::*;

#[component]
pub fn EntryRow(
    entry: Entry,
    projects: Vec<Project>,
    on_delete: EventHandler<Entry>,
    on_edit: EventHandler<Entry>,
) -> Element {
    let date_part = &entry.start_time[..10];
    let start_part = &entry.start_time[11..16];
    let end_part = entry.end_time.as_ref().map(|e| &e[11..16]).unwrap_or("now");

    let project_name = entry.project_id.and_then(|pid| {
        projects
            .iter()
            .find(|p| p.id == pid)
            .map(|p| p.name.as_str())
    });
    let delete_entry = entry.clone();
    let edit_entry = entry.clone();

    rsx! {
        div { class: "entry",
            div { class: "entry-text",
                span { class: "entry-title", "{entry.title}" }
                span { class: "entry-time",
                    "{date_part}  {start_part} \u{2013} {end_part}"
                    if let Some(pn) = project_name {
                        span { " \u{00B7} {pn}" }
                    }
                }
            }
            div { class: "entry-actions",
                if entry.end_time.is_none() {
                    span { class: "dot dot-on" }
                }
                button {
                    class: "entry-edit",
                    aria_label: "Edit entry",
                    onclick: move |_| on_edit.call(edit_entry.clone()),
                    "✎"
                }
                button {
                    class: "entry-delete",
                    aria_label: "Delete entry",
                    onclick: move |_| on_delete.call(delete_entry.clone()),
                    "\u{00D7}"
                }
            }
        }
    }
}
