use dioxus::prelude::*;

#[component]
pub fn TimerDisplay(elapsed_seconds: Signal<u64>) -> Element {
    let formatted = use_memo(move || {
        let total = *elapsed_seconds.read();
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let seconds = total % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    });

    rsx! {
        div {
            class: "font-mono text-5xl tabular-nums tracking-tight text-zinc-900 select-none",
            "{formatted}"
        }
    }
}
