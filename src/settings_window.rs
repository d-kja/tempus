use crate::bridge;
use dioxus::prelude::*;
use std::collections::HashMap;

static CSS: Asset = asset!("/assets/app.css");

#[component]
pub fn SettingsWindow() -> Element {
    let mut settings = use_signal(|| HashMap::new());
    let mut always_on_top = use_signal(|| true);
    let mut loaded = use_signal(|| false);

    use_effect(move || {
        if *loaded.read() {
            return;
        }
        loaded.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(s) = bridge::get_settings().await {
                let aot = s
                    .get("always_on_top")
                    .cloned()
                    .unwrap_or("true".into())
                    == "true";
                settings.set(s);
                always_on_top.set(aot);
            }
        });
    });

    let on_toggle = move |_| {
        let new = !*always_on_top.read();
        always_on_top.set(new);
        let mut s = settings.read().clone();
        s.insert("always_on_top".into(), new.to_string());
        settings.set(s.clone());
        spawn(async move {
            let _ = bridge::set_always_on_top(new).await;
            let _ = bridge::update_settings(s).await;
        });
    };

    let on_close = move |_| {
        spawn(async move {
            let _ = bridge::close_current_window().await;
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CSS }
        div { class: "settings-window",
            div { class: "settings-window-header",
                h2 { class: "settings-window-title", "Settings" }
                button {
                    class: "settings-window-close",
                    onclick: on_close,
                    "\u{00D7}"
                }
            }
            div { class: "settings-window-body",
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
}