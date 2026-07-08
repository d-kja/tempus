use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Timer,
    Setup,
}

#[component]
pub fn Navigation(current: Signal<Page>) -> Element {
    let tabs = [(Page::Timer, "Timer"), (Page::Setup, "Setup")];

    rsx! {
        nav { class: "nav",
            for (page, label) in tabs.iter() {
                button {
                    class: if *current.read() == *page { "nav-btn nav-btn-on" } else { "nav-btn" },
                    onclick: {
                        let p = page.clone();
                        move |_| current.set(p)
                    },
                    if *current.read() == *page {
                        span { class: "nav-indicator" }
                    }
                    "{label}"
                }
            }
        }
    }
}