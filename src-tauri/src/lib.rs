mod commands;
mod db;
mod models;

use crate::commands::{entries, projects, settings, export as export_mod};
use db::Database;
use std::collections::HashMap;
use tauri::{Emitter, Manager, State, WebviewWindow, WebviewWindowBuilder, WebviewUrl};

#[tauri::command]
fn start_entry(
    app: tauri::AppHandle,
    db: State<Database>,
    title: String,
    description: Option<String>,
    project_id: Option<i64>,
) -> Result<crate::models::Entry, String> {
    let entry = entries::start_entry_impl(&db, &title, description.as_deref(), project_id)?;
    let _ = app.emit("entry-started", &entry);
    Ok(entry)
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
fn resume_entry(db: State<Database>, id: i64) -> Result<crate::models::Entry, String> {
    entries::resume_entry_impl(&db, id)
}

#[tauri::command]
fn delete_entry(db: State<Database>, id: i64) -> Result<(), String> {
    entries::delete_entry_impl(&db, id)
}

#[tauri::command]
fn clear_all_entries(db: State<Database>) -> Result<(), String> {
    entries::clear_all_entries_impl(&db)
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
        .set_size(tauri::LogicalSize::new(width, height))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_always_on_top(app: tauri::AppHandle, always: bool) -> Result<(), String> {
    if let Some(main) = app.get_webview_window("main") {
        main.set_always_on_top(always).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_settings(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("settings") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("/index.html?settings".into()))
        .title("Settings")
        .inner_size(360.0, 520.0)
        .decorations(false)
        .resizable(true)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn open_new_entry(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(existing) = app.get_webview_window("new-entry") {
        let _ = existing.show();
        let _ = existing.set_focus();
        return Ok(());
    }
    WebviewWindowBuilder::new(&app, "new-entry", WebviewUrl::App("/index.html?new-entry".into()))
        .title("New Entry")
        .inner_size(320.0, 200.0)
        .decorations(false)
        .resizable(false)
        .transparent(true)
        .shadow(true)
        .always_on_top(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn close_current_window(window: WebviewWindow) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
fn set_window_position(window: WebviewWindow, x: f64, y: f64) -> Result<(), String> {
    window
        .set_position(tauri::LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            let db = Database::new(app_dir).map_err(|e| format!("DB init failed: {}", e))?;

            let win = app.get_webview_window("main").expect("no main window");

            commands::window::restore_always_on_top(&db, &win)?;

            // Note: on Wayland (Hyprland) clients cannot set window position —
            // the compositor owns placement. Position is controlled via a
            // Hyprland window rule, e.g. in hyprland.conf:
            //   windowrulev2 = move 100% 0%, class:^(com.d-kja-hours)$
            //   windowrulev2 = float, class:^(com.d-kja-hours)$
            // On X11 we still save/restore last position below.
            #[cfg(not(target_os = "linux"))]
            {
                let saved_pos = commands::window::load_window_position(&db);
                if let Some((x, y)) = saved_pos {
                    let _ = win.set_position(tauri::LogicalPosition::new(x, y));
                }
            }

            let handle = app.handle().clone();
            win.on_window_event(move |event| {
                if let tauri::WindowEvent::Moved(position) = event {
                    let db_state = handle.state::<Database>();
                    let _ = commands::window::save_window_position(
                        &db_state,
                        position.x as f64,
                        position.y as f64,
                    );
                }
            });

            app.manage(db);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_entry,
            stop_entry,
            resume_entry,
            get_active_entry,
            get_entries,
            update_entry,
            delete_entry,
            clear_all_entries,
            create_project,
            get_projects,
            delete_project,
            get_settings_db,
            update_settings_db,
            export_markdown,
            set_window_size,
            set_always_on_top,
            set_window_position,
            open_settings,
            open_new_entry,
            close_current_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
