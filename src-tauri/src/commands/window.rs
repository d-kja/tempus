use crate::db::Database;
use std::collections::HashMap;
use tauri::WebviewWindow;

pub fn save_window_position(db: &Database, x: f64, y: f64) -> Result<(), String> {
    let mut map = HashMap::new();
    map.insert("window_x".to_string(), x.to_string());
    map.insert("window_y".to_string(), y.to_string());
    crate::commands::settings::update_settings_impl(db, &map)
}

pub fn load_window_position(db: &Database) -> Option<(f64, f64)> {
    let settings = crate::commands::settings::get_settings_impl(db).ok()?;
    let x = settings.get("window_x")?.parse::<f64>().ok()?;
    let y = settings.get("window_y")?.parse::<f64>().ok()?;
    if (x - 0.0).abs() > f64::EPSILON || (y - 0.0).abs() > f64::EPSILON {
        Some((x, y))
    } else {
        None
    }
}

pub fn restore_always_on_top(db: &Database, window: &WebviewWindow) -> Result<(), String> {
    let settings = crate::commands::settings::get_settings_impl(db)?;
    if let Some(val) = settings.get("always_on_top") {
        if val == "true" {
            window
                .set_always_on_top(true)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
