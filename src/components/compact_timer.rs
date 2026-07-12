use crate::bridge;
use crate::components::timer_display::TimerDisplay;
use crate::state::{AppState, TimerState};
use dioxus::prelude::*;
use gloo_timers::callback::Interval;

#[component]
pub fn CompactTimer() -> Element {
    let state = use_context::<AppState>();
    let mut elapsed = use_signal(|| 0u64);
    let mut interval_handle = use_signal(|| Option::<Interval>::None);
    let mut poll_handle = use_signal(|| Option::<Interval>::None);
    let is_running = use_memo(move || matches!(*state.timer.read(), TimerState::Running(_)));
    let is_paused = use_memo(move || matches!(*state.timer.read(), TimerState::Stopped(_)));

    use_effect(move || {
        if *is_running.read() {
            if interval_handle.read().is_none() {
                let mut elapsed_copy = elapsed;
                let interval = Interval::new(1000, move || {
                    let current = *elapsed_copy.read();
                    elapsed_copy.set(current + 1);
                });
                *interval_handle.write() = Some(interval);
            }
        } else {
            *interval_handle.write() = None;
            if matches!(*state.timer.read(), TimerState::Idle) {
                elapsed.set(0);
            }
        }
    });

    use_effect(move || {
        if poll_handle.read().is_none() {
            let timer = state.timer;
            let interval = Interval::new(2000, move || {
                if matches!(*timer.read(), TimerState::Idle) {
                    let mut current_timer = timer;
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(Some(entry)) = bridge::get_active_entry().await {
                            current_timer.set(TimerState::Running(entry));
                        }
                    });
                }
            });
            *poll_handle.write() = Some(interval);
        }
    });

    let on_start = move |_| {
        spawn(async move {
            let _ = bridge::open_new_entry().await;
        });
    };

    let on_settings = move |_| {
        spawn(async move {
            let _ = bridge::open_settings().await;
        });
    };

    let on_pause = {
        let mut state = state.clone();
        move |_| {
            spawn(async move {
                if let Ok(Some(entry)) = bridge::stop_entry().await {
                    state.timer.set(TimerState::Stopped(entry));
                }
            });
        }
    };

    let on_stop = {
        let mut state = state.clone();
        move |_| {
            spawn(async move {
                let _ = bridge::stop_entry().await;
                state.timer.set(TimerState::Idle);
            });
        }
    };

    let on_resume = {
        let mut state = state.clone();
        move |_| {
            if let TimerState::Stopped(entry) = state.timer.read().clone() {
                spawn(async move {
                    if let Ok(entry) = bridge::resume_entry(entry.id).await {
                        elapsed.set(0);
                        state.timer.set(TimerState::Running(entry));
                    }
                });
            }
        }
    };

    rsx! {
        div { class: "compact",
            div { class: "compact-surface",
                if *is_running.read() {
                    div { id: "timer-pill", class: "timer-pill timer-pill-running",
                        div { class: "timer-running-content",
                            div { class: "timer-handle",
                                span { class: "timer-handle-dot" }
                            }
                            svg {
                                class: "timer-clock",
                                view_box: "0 0 14 14",
                                path {
                                    d: "M7.29053 6.88037V4.375q0-.12305-.08545-.20508-.08203-.08545-.20508-.08545-.12305 0-.2085.08545-.08203.08203-.08203.20508v2.55322q0 .08887.03076.17432.03418.08203.10938.16064l2.07129 2.07129q.08203.08203.19824.08887.11963.00342.21533-.08887.09229-.0957.09229-.20849 0-.11279-.09229-.20508L7.29053 6.88037ZM7.00342 12.25q-1.09033 0-2.05078-.41357-0.95703-.41357-1.66797-1.1211-0.70752-.70752-1.1211-1.66455-.41357-.96045-.41357-2.04736 0-1.09033.41357-2.04737.41357-.96045 1.1211-1.66796.70752-.71094 1.66455-1.12452.96045-.41357 2.04736-.41357 1.09033 0 2.04737.41357.96045.41357 1.66796 1.1211.71094.70752 1.12452 1.66797.41357.95703.41357 2.04394 0 1.09033-.41357 2.05078-.41357.95703-1.1211 1.66797-.70752.70752-1.66797 1.1211-.95703.41357-2.04394.41357Zm-.00342-.58447q1.93799 0 3.30176-1.36377 1.36377-1.36377 1.36377-3.30176 0-1.93799-1.36377-3.30176-1.36377-1.36377-3.30176-1.36377-1.93799 0-3.30176 1.36377-1.36377 1.36377-1.36377 3.30176 0 1.93799 1.36377 3.30176 1.36377 1.36377 3.30176 1.36377Z",
                                    fill: "#D7D7D0B8"
                                }
                            }
                            TimerDisplay { elapsed_seconds: elapsed }
                            span { class: "timer-separator" }
                            button {
                                class: "timer-control timer-control-pause",
                                onclick: on_pause,
                                aria_label: "Pause timer",
                                svg {
                                    class: "timer-icon",
                                    view_box: "0 0 14 14",
                                    path {
                                        d: "M8.45947 10.5q-.23584 0-.41015-.17432-.17432-.17432-.17432-.41015V4.08447q0-.23584.17432-.41015.17431-.17432.41015-.17432h1.45606q.23584 0 .41015.17432.17432.17431.17432.41015v5.83106q0 .23584-.17432.41015-.17431.17432-.41015.17432H8.45947Zm-4.375 0q-.23584 0-.41015-.17432-.17432-.17432-.17432-.41015V4.08447q0-.23584.17432-.41015.17431-.17432.41015-.17432h1.45606q.23584 0 .41015.17432.17432.17431.17432.41015v5.83106q0 .23584-.17432.41015-.17431.17432-.41015.17432H4.08447Zm4.375-.58447h1.45606V4.08447H8.45947v5.83106Zm-4.375 0h1.45606V4.08447H4.08447v5.83106Z",
                                        fill: "#111113"
                                    }
                                }
                            }
                            button {
                                class: "timer-control timer-control-stop",
                                onclick: on_stop,
                                aria_label: "Stop timer",
                                svg {
                                    class: "timer-icon",
                                    view_box: "0 0 14 14",
                                    path {
                                        d: "M4.08447 8.97559V5.02441q0-.38965.27344-.66308.27686-.27686.6665-.27686h3.95118q.38965 0 .66308.27686.27686.27343.27686.66308v3.95118q0 .38965-.27686.6665-.27343.27344-.66308.27344H5.02441q-.38964 0-.6665-.27344-.27344-.27685-.27344-.6665Zm.93994.35888h3.95118q.15723 0 .25634-.09912.10254-.10254.10254-.25976V5.02441q0-.15723-.10254-.25634-.09911-.10254-.25634-.10254H5.02441q-.15723 0-.25976.10254-.09912.09911-.09912.25634v3.95118q0 .15722.09912.25976.10253.09912.25976.09912Z",
                                        fill: "#F8F8F4"
                                    }
                                }
                            }
                        }
                    }
                } else if *is_paused.read() {
                    div { id: "timer-pill", class: "timer-pill timer-pill-mini",
                        TimerDisplay { elapsed_seconds: elapsed }
                        button {
                            class: "timer-state-action",
                            onclick: on_resume,
                            aria_label: "Resume timer",
                            span { class: "timer-play-icon", "▶" }
                            "Resume"
                        }
                    }
                } else {
                    div { id: "timer-pill", class: "timer-pill timer-pill-mini",
                        TimerDisplay { elapsed_seconds: elapsed }
                        button {
                            class: "timer-control timer-control-settings",
                            onclick: on_settings,
                            aria_label: "Open settings",
                            "⚙"
                        }
                        button {
                            class: "timer-state-action",
                            onclick: on_start,
                            aria_label: "Start timer",
                            span { class: "timer-play-icon", "▶" }
                            "Start"
                        }
                    }
                }
            }
        }
    }
}
