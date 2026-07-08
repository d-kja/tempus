use crate::bridge::Entry;
use dioxus::prelude::*;

#[component]
pub fn EntryRow(entry: Entry) -> Element {
    let date_part = &entry.start_time[..10];
    let start_part = &entry.start_time[11..16];
    let end_part = entry
        .end_time
        .as_ref()
        .map(|e| &e[11..16])
        .unwrap_or("...");

    rsx! {
        div {
            class: "flex items-center justify-between px-3 py-2 border-b border-zinc-200 text-xs",
            div { class: "flex flex-col",
                span { class: "text-zinc-900 font-medium truncate max-w-[180px]", "{entry.title}" }
                span { class: "text-zinc-500",
                    "{date_part}  {start_part} - {end_part}"
                }
            }
        }
    }
}
