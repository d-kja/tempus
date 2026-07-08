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
        div { class: "page",
            section { class: "section",
                h3 { class: "section-label", "Display" }
                button {
                    class: "toggle-row",
                    onclick: on_toggle,
                    span { class: "toggle-label", "Always on top" }
                    span {
                        class: if *always_on_top.read() { "toggle toggle-on" } else { "toggle toggle-off" },
                        span {
                            class: if *always_on_top.read() { "toggle-knob toggle-knob-on" } else { "toggle-knob toggle-knob-off" }
                        }
                    }
                }
                p { class: "helper-text", "Keep the timer visible above all other windows." }
            }
        }
    }
}