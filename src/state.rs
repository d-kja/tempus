use crate::bridge::{Entry, Project};
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TimerState {
    Idle,
    Running(Entry),
    Stopped(Entry),
}

#[derive(Clone)]
pub struct AppState {
    pub timer: Signal<TimerState>,
    pub entries: Signal<Vec<Entry>>,
    pub projects: Signal<Vec<Project>>,
    pub is_expanded: Signal<bool>,
    pub settings: Signal<HashMap<String, String>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            timer: Signal::new(TimerState::Idle),
            entries: Signal::new(Vec::new()),
            projects: Signal::new(Vec::new()),
            is_expanded: Signal::new(false),
            settings: Signal::new(HashMap::new()),
        }
    }
}
