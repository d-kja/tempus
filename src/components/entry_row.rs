use crate::bridge::Entry;
use dioxus::prelude::*;

#[component]
pub fn EntryRow(entry: Entry, on_delete: EventHandler<i64>) -> Element {
    let eid = entry.id;
    let date_part = &entry.start_time[..10];
    let start_part = &entry.start_time[11..16];
    let end_part = entry
        .end_time
        .as_ref()
        .map(|e| &e[11..16])
        .unwrap_or("now");

    rsx! {
        div { class: "entry",
            div { class: "entry-text",
                span { class: "entry-title", "{entry.title}" }
                span { class: "entry-time",
                    "{date_part}  {start_part} \u{2013} {end_part}"
                }
            }
            div { class: "entry-actions",
                if entry.end_time.is_none() {
                    span { class: "dot dot-on" }
                }
                button {
                    class: "entry-delete",
                    onclick: move |_| on_delete.call(eid),
                    "\u{00D7}"
                }
            }
        }
    }
}
