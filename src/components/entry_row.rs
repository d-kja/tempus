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
        .unwrap_or("now");

    rsx! {
        div { class: "flex items-center justify-between px-4 py-2.5",
            div { class: "flex flex-col min-w-0",
                span { class: "text-xs text-zinc-900 font-medium truncate max-w-[180px]", "{entry.title}" }
                span { class: "text-[10px] text-zinc-500 font-mono tabular-nums",
                    "{date_part}  {start_part} \u{2013} {end_part}"
                }
            }
            if entry.end_time.is_none() {
                span { class: "w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0" }
            }
        }
    }
}