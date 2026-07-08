use crate::bridge;
use dioxus::prelude::*;

#[component]
pub fn NewEntryWindow() -> Element {
    let mut title = use_signal(|| "Untitled".to_string());

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        div { class: "new-entry",
            h3 { class: "new-entry-label", "What are you working on?" }
            input {
                class: "input",
                value: "{title}",
                autofocus: true,
                oninput: move |e| title.set(e.value()),
                onkeydown: move |e| {
                    if e.key() == Key::Enter {
                        let t = title.read().clone();
                        spawn(async move {
                            let _ = bridge::start_entry(t, None, None).await;
                            let _ = bridge::close_current_window().await;
                        });
                    }
                }
            }
            div { class: "new-entry-actions",
                button {
                    class: "btn btn-outline btn-sm",
                    onclick: move |_| {
                        spawn(async move {
                            let _ = bridge::close_current_window().await;
                        });
                    },
                    "Cancel"
                }
                button {
                    class: "btn btn-primary btn-sm",
                    onclick: move |_| {
                        let t = title.read().clone();
                        spawn(async move {
                            let _ = bridge::start_entry(t, None, None).await;
                            let _ = bridge::close_current_window().await;
                        });
                    },
                    "Start"
                }
            }
        }
    }
}
