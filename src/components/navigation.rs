use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Timer,
    Setup,
    Settings,
}

#[component]
pub fn Navigation(current: Signal<Page>) -> Element {
    let tabs = [
        (Page::Timer, "Timer"),
        (Page::Setup, "Setup"),
        (Page::Settings, "Settings"),
    ];

    rsx! {
        nav {
            class: "flex border-t border-zinc-200 bg-zinc-50",
            for (page, label) in tabs.iter() {
                button {
                    class: if *current.read() == *page {
                        "flex-1 py-2 text-xs font-medium text-zinc-900 border-t-2 border-zinc-800"
                    } else {
                        "flex-1 py-2 text-xs text-zinc-500 hover:text-zinc-700 hover:bg-zinc-100 transition-colors"
                    },
                    onclick: {
                        let p = page.clone();
                        move |_| current.set(p)
                    },
                    "{label}"
                }
            }
        }
    }
}
