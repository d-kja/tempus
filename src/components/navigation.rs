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
        nav { class: "flex border-t border-zinc-100 bg-zinc-50",
            for (page, label) in tabs.iter() {
                button {
                    class: if *current.read() == *page {
                        "flex-1 py-2.5 text-xs font-medium text-zinc-900 relative"
                    } else {
                        "flex-1 py-2.5 text-xs text-zinc-500 hover:text-zinc-800 transition-colors"
                    },
                    onclick: {
                        let p = page.clone();
                        move |_| current.set(p)
                    },
                    if *current.read() == *page {
                        span { class: "absolute bottom-0 left-1/2 -translate-x-1/2 w-6 h-0.5 bg-zinc-900 rounded-full" }
                    }
                    "{label}"
                }
            }
        }
    }
}