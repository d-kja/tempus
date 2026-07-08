use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub project_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Serialize)]
struct StartEntryArgs {
    title: String,
    description: Option<String>,
    project_id: Option<i64>,
}

#[derive(Serialize)]
struct GetEntriesArgs {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Serialize)]
struct SetWindowSizeArgs {
    width: f64,
    height: f64,
}

fn from_value<T: for<'de> Deserialize<'de>>(val: JsValue) -> Result<T, String> {
    serde_wasm_bindgen::from_value(val).map_err(|e| format!("deserialize: {}", e))
}

pub async fn start_entry(
    title: String,
    description: Option<String>,
    project_id: Option<i64>,
) -> Result<Entry, String> {
    let args = StartEntryArgs {
        title,
        description,
        project_id,
    };
    let js_args = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;
    let val = invoke("start_entry", js_args).await;
    from_value(val)
}

pub async fn stop_entry() -> Result<Option<Entry>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let val = invoke("stop_entry", args).await;
    from_value(val)
}

pub async fn resume_entry(id: i64) -> Result<Entry, String> {
    let wrapper = serde_json::json!({ "id": id });
    let args = serde_wasm_bindgen::to_value(&wrapper).unwrap();
    from_value(invoke("resume_entry", args).await)
}

pub async fn get_active_entry() -> Result<Option<Entry>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let val = invoke("get_active_entry", args).await;
    from_value(val)
}

pub async fn get_entries(limit: Option<i64>, offset: Option<i64>) -> Result<Vec<Entry>, String> {
    let args = GetEntriesArgs { limit, offset };
    let js_args = serde_wasm_bindgen::to_value(&args).unwrap();
    let val = invoke("get_entries", js_args).await;
    from_value(val)
}

pub async fn delete_entry(id: i64) -> Result<(), String> {
    let wrapper = serde_json::json!({ "id": id });
    let args = serde_wasm_bindgen::to_value(&wrapper).unwrap();
    let _ = invoke("delete_entry", args).await;
    Ok(())
}

pub async fn create_project(name: String) -> Result<Project, String> {
    let wrapper = serde_json::json!({ "name": name });
    let args = serde_wasm_bindgen::to_value(&wrapper).unwrap();
    let val = invoke("create_project", args).await;
    from_value(val)
}

pub async fn get_projects() -> Result<Vec<Project>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let val = invoke("get_projects", args).await;
    from_value(val)
}

pub async fn delete_project(id: i64) -> Result<(), String> {
    let wrapper = serde_json::json!({ "id": id });
    let args = serde_wasm_bindgen::to_value(&wrapper).unwrap();
    let _ = invoke("delete_project", args).await;
    Ok(())
}

pub async fn get_settings() -> Result<std::collections::HashMap<String, String>, String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let val = invoke("get_settings_db", args).await;
    from_value(val)
}

pub async fn update_settings(
    settings: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&settings).unwrap();
    let _ = invoke("update_settings_db", args).await;
    Ok(())
}

pub async fn export_markdown(path: String) -> Result<(), String> {
    let wrapper = serde_json::json!({ "path": path });
    let args = serde_wasm_bindgen::to_value(&wrapper).unwrap();
    let _val: Result<String, String> = from_value(invoke("export_markdown", args).await);
    Ok(())
}

pub async fn set_window_size(width: f64, height: f64) -> Result<(), String> {
    let args = SetWindowSizeArgs { width, height };
    let js_args = serde_wasm_bindgen::to_value(&args).map_err(|e| e.to_string())?;
    let _ = invoke("set_window_size", js_args).await;
    Ok(())
}

pub async fn set_always_on_top(always: bool) -> Result<(), String> {
    let wrapper = serde_json::json!({ "always": always });
    let args = serde_wasm_bindgen::to_value(&wrapper).unwrap();
    let _ = invoke("set_always_on_top", args).await;
    Ok(())
}

pub async fn open_settings() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let _ = invoke("open_settings", args).await;
    Ok(())
}

pub async fn close_current_window() -> Result<(), String> {
    let args = serde_wasm_bindgen::to_value(&()).unwrap();
    let _ = invoke("close_current_window", args).await;
    Ok(())
}
