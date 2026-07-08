use crate::bridge::Entry;
use dioxus::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Idle,
    Running(Entry),
    #[allow(dead_code)]
    Stopped(Entry),
}

#[derive(Clone)]
pub struct AppState {
    pub timer: Signal<TimerState>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            timer: Signal::new(TimerState::Idle),
        }
    }
}