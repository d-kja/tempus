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
        div { class: "flex flex-col gap-4 p-4",
            section {
                h3 { class: "text-sm font-medium text-zinc-900 mb-2", "Display" }
                label { class: "flex items-center justify-between py-2 cursor-pointer",
                    span { class: "text-sm text-zinc-700", "Always on top" }
                    button {
                        class: if *always_on_top.read() {
                            "w-10 h-5 rounded-full bg-zinc-800 transition-colors relative"
                        } else {
                            "w-10 h-5 rounded-full bg-zinc-300 transition-colors relative"
                        },
                        onclick: on_toggle,
                        div {
                            class: if *always_on_top.read() {
                                "absolute right-0.5 top-0.5 w-4 h-4 rounded-full bg-white transition-all"
                            } else {
                                "absolute left-0.5 top-0.5 w-4 h-4 rounded-full bg-white transition-all"
                            }
                        }
                    }
                }
            }
        }
    }
}
