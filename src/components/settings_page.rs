use crate::bridge;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    let mut state = use_context::<AppState>();

    let always_on_top = use_memo(move || {
        state
            .settings
            .read()
            .get("always_on_top")
            .cloned()
            .unwrap_or("true".into())
            == "true"
    });

    let on_toggle = move |_| {
        let new = !*always_on_top.read();
        let mut s = state.settings.read().clone();
        s.insert("always_on_top".into(), new.to_string());
        state.settings.set(s.clone());
        spawn(async move {
            let _ = bridge::set_always_on_top(new).await;
            let _ = bridge::update_settings(s).await;
        });
    };

    rsx! {
        div { class: "flex flex-col gap-6 p-4",
            section { class: "flex flex-col gap-2",
                h3 { class: "text-[10px] font-medium uppercase tracking-[0.18em] text-zinc-400", "Display" }
                button {
                    class: "flex items-center justify-between w-full py-2 group",
                    onclick: on_toggle,
                    span { class: "text-sm text-zinc-800", "Always on top" }
                    span {
                        class: if *always_on_top.read() {
                            "w-10 h-5 rounded-full bg-zinc-900 relative transition-colors"
                        } else {
                            "w-10 h-5 rounded-full bg-zinc-300 relative transition-colors"
                        },
                        span {
                            class: if *always_on_top.read() {
                                "absolute top-0.5 right-0.5 w-4 h-4 rounded-full bg-zinc-50 transition-all"
                            } else {
                                "absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-all"
                            }
                        }
                    }
                }
                p { class: "text-[11px] text-zinc-500", "Keep the timer visible above all other windows." }
            }
        }
    }
}