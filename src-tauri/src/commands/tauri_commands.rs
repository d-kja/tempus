use crate::commands::{entries, projects, settings, export as export_mod, window as window_mod};
use crate::db::Database;
use std::collections::HashMap;
use tauri::{State, WebviewWindow};

#[tauri::command]
fn start_entry(
    db: State<Database>,
    title: String,
    description: Option<String>,
    project_id: Option<i64>,
) -> Result<crate::models::Entry, String> {
    entries::start_entry_impl(&db, &title, description.as_deref(), project_id)
}

#[tauri::command]
fn stop_entry(db: State<Database>) -> Result<Option<crate::models::Entry>, String> {
    entries::stop_entry_impl(&db)
}

#[tauri::command]
fn get_active_entry(db: State<Database>) -> Result<Option<crate::models::Entry>, String> {
    entries::get_active_entry_impl(&db)
}

#[tauri::command]
fn get_entries(
    db: State<Database>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<crate::models::Entry>, String> {
    entries::get_entries_impl(&db, limit.unwrap_or(50), offset.unwrap_or(0))
}

#[tauri::command]
fn update_entry(
    db: State<Database>,
    id: i64,
    title: Option<String>,
    description: Option<Option<String>>,
    project_id: Option<Option<i64>>,
) -> Result<crate::models::Entry, String> {
    entries::update_entry_impl(
        &db,
        id,
        title.as_deref(),
        description.as_ref().map(|d| d.as_deref()),
        project_id,
    )
}

#[tauri::command]
fn delete_entry(db: State<Database>, id: i64) -> Result<(), String> {
    entries::delete_entry_impl(&db, id)
}

#[tauri::command]
fn create_project(db: State<Database>, name: String) -> Result<crate::models::Project, String> {
    projects::create_project_impl(&db, &name)
}

#[tauri::command]
fn get_projects(db: State<Database>) -> Result<Vec<crate::models::Project>, String> {
    projects::get_projects_impl(&db)
}

#[tauri::command]
fn delete_project(db: State<Database>, id: i64) -> Result<(), String> {
    projects::delete_project_impl(&db, id)
}

#[tauri::command]
fn get_settings_db(db: State<Database>) -> Result<HashMap<String, String>, String> {
    settings::get_settings_impl(&db)
}

#[tauri::command]
fn update_settings_db(
    db: State<Database>,
    new_settings: HashMap<String, String>,
) -> Result<(), String> {
    settings::update_settings_impl(&db, &new_settings)
}

#[tauri::command]
fn export_markdown(db: State<Database>, path: String) -> Result<(), String> {
    export_mod::export_markdown_impl(&db, &path)
}

#[tauri::command]
fn set_window_size(window: WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    window
        .set_size(tauri::PhysicalSize::new(width as u32, height as u32))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_always_on_top(window: WebviewWindow, always: bool) -> Result<(), String> {
    window
        .set_always_on_top(always)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_window_position(window: WebviewWindow, x: f64, y: f64) -> Result<(), String> {
    window
        .set_position(tauri::PhysicalPosition::new(x as i32, y as i32))
        .map_err(|e| e.to_string())
}
