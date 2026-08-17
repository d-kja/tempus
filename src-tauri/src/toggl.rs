use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

const TOGGL_API_BASE: &str = "https://api.track.toggl.com/api/v9";
const CREATED_WITH: &str = "tempus";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TogglUser {
    pub default_workspace_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TogglProject {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TogglTimeEntry {
    pub id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTimeEntry {
    pub description: String,
    pub start: String,
    pub stop: String,
    pub duration: i64,
    pub project_id: Option<i64>,
}

pub trait TogglApi {
    fn get_me(&self) -> Result<TogglUser, String>;
    fn list_projects(&self, workspace_id: i64) -> Result<Vec<TogglProject>, String>;
    fn create_project(&self, workspace_id: i64, name: &str) -> Result<TogglProject, String>;
    fn create_time_entry(
        &self,
        workspace_id: i64,
        entry: &NewTimeEntry,
    ) -> Result<TogglTimeEntry, String>;
}

pub struct TogglClient {
    agent: ureq::Agent,
    api_token: String,
}

impl TogglClient {
    pub fn new(api_token: impl Into<String>) -> Result<Self, String> {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(30))
            .user_agent("tempus/0.1")
            .build();
        Ok(Self {
            agent,
            api_token: api_token.into(),
        })
    }

    fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<String, String> {
        let url = format!("{TOGGL_API_BASE}{path}");
        let mut last_error = String::new();

        for attempt in 0..3 {
            let request = self
                .agent
                .request(method, &url)
                .set("Content-Type", "application/json")
                .set("Authorization", &basic_auth_header(&self.api_token));

            let result = match body.clone() {
                Some(payload) => request.send_json(payload),
                None => request.call(),
            };

            match result {
                Ok(response) => {
                    return response
                        .into_string()
                        .map_err(|e| format!("failed to read Toggl response: {e}"));
                }
                Err(ureq::Error::Status(code, response)) => {
                    let text = response.into_string().unwrap_or_default();
                    last_error = map_toggl_error(code, &text);
                    if code == 429 || (500..600).contains(&code) {
                        std::thread::sleep(Duration::from_secs(2 * (attempt + 1) as u64));
                        continue;
                    }
                    return Err(last_error);
                }
                Err(error) => return Err(format!("Toggl request failed: {error}")),
            }
        }

        Err(last_error)
    }
}

impl TogglApi for TogglClient {
    fn get_me(&self) -> Result<TogglUser, String> {
        let body = self.request("GET", "/me", None)?;
        let parsed: MeResponse =
            serde_json::from_str(&body).map_err(|e| format!("invalid Toggl /me response: {e}"))?;
        Ok(TogglUser {
            default_workspace_id: parsed.default_workspace_id,
        })
    }

    fn list_projects(&self, workspace_id: i64) -> Result<Vec<TogglProject>, String> {
        let body = self.request("GET", &format!("/workspaces/{workspace_id}/projects"), None)?;
        let parsed: Vec<ProjectResponse> = serde_json::from_str(&body)
            .map_err(|e| format!("invalid Toggl projects response: {e}"))?;
        Ok(parsed
            .into_iter()
            .filter(|project| project.active.unwrap_or(true))
            .map(|project| TogglProject {
                id: project.id,
                name: project.name,
            })
            .collect())
    }

    fn create_project(&self, workspace_id: i64, name: &str) -> Result<TogglProject, String> {
        let body = self.request(
            "POST",
            &format!("/workspaces/{workspace_id}/projects"),
            Some(json!({
                "name": name,
                "active": true,
                "is_private": true,
            })),
        )?;
        let parsed: ProjectResponse = serde_json::from_str(&body)
            .map_err(|e| format!("invalid Toggl create project response: {e}"))?;
        // Toggl's auth layer is eventually consistent after creating entities.
        std::thread::sleep(Duration::from_secs(2));
        Ok(TogglProject {
            id: parsed.id,
            name: parsed.name,
        })
    }

    fn create_time_entry(
        &self,
        workspace_id: i64,
        entry: &NewTimeEntry,
    ) -> Result<TogglTimeEntry, String> {
        let body = self.request(
            "POST",
            &format!("/workspaces/{workspace_id}/time_entries"),
            Some(build_time_entry_body(workspace_id, entry)),
        )?;
        let parsed: TimeEntryResponse = serde_json::from_str(&body)
            .map_err(|e| format!("invalid Toggl create time entry response: {e}"))?;
        Ok(TogglTimeEntry { id: parsed.id })
    }
}

#[derive(Debug, Deserialize)]
struct MeResponse {
    default_workspace_id: i64,
}

#[derive(Debug, Deserialize)]
struct ProjectResponse {
    id: i64,
    name: String,
    #[serde(default)]
    active: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TimeEntryResponse {
    id: i64,
}

pub fn toggl_description(title: &str, description: Option<&str>) -> String {
    match description.map(str::trim).filter(|value| !value.is_empty()) {
        Some(details) => format!("{title} - {details}"),
        None => title.to_string(),
    }
}

pub fn sql_datetime_to_rfc3339(value: &str) -> Result<String, String> {
    Ok(sql_datetime_to_zoned(value)?
        .timestamp()
        .strftime("%Y-%m-%dT%H:%M:%SZ")
        .to_string())
}

pub fn duration_secs(start_sql: &str, end_sql: &str) -> Result<i64, String> {
    let start = sql_datetime_to_zoned(start_sql)?;
    let end = sql_datetime_to_zoned(end_sql)?;
    Ok(end.timestamp().as_second() - start.timestamp().as_second())
}

fn sql_datetime_to_zoned(value: &str) -> Result<jiff::Zoned, String> {
    if value.len() < 19 {
        return Err("invalid timestamp format".into());
    }
    let iso = format!("{}T{}", &value[0..10], &value[11..19]);
    let dt: jiff::civil::DateTime = iso
        .parse()
        .map_err(|e| format!("invalid timestamp '{value}': {e}"))?;
    jiff::tz::TimeZone::system()
        .to_zoned(dt)
        .map_err(|e| format!("invalid local time '{value}': {e}"))
}

pub fn build_time_entry_body(workspace_id: i64, entry: &NewTimeEntry) -> serde_json::Value {
    let mut body = json!({
        "created_with": CREATED_WITH,
        "description": entry.description,
        "start": entry.start,
        "stop": entry.stop,
        "duration": entry.duration,
        "workspace_id": workspace_id,
    });
    if let Some(project_id) = entry.project_id {
        body["project_id"] = json!(project_id);
    }
    body
}

fn basic_auth_header(token: &str) -> String {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    format!("Basic {}", STANDARD.encode(format!("{token}:api_token")))
}

fn map_toggl_error(status: u16, body: &str) -> String {
    let trimmed = body.trim().trim_matches('"');
    match status {
        401 | 403 => "Invalid Toggl API token.".into(),
        429 => "Toggl rate limit reached. Try again in a minute.".into(),
        402 => "Toggl API quota exceeded. Try again later.".into(),
        _ if trimmed.is_empty() => format!("Toggl API error ({status})."),
        _ => format!("Toggl API error ({status}): {trimmed}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggl_description_uses_title_and_optional_details() {
        assert_eq!(toggl_description("Coding", None), "Coding");
        assert_eq!(toggl_description("Coding", Some("")), "Coding");
        assert_eq!(
            toggl_description("Coding", Some("  Handoff notes  ")),
            "Coding - Handoff notes"
        );
    }

    #[test]
    fn test_sql_datetime_to_rfc3339() {
        let rfc = sql_datetime_to_rfc3339("2026-08-17 10:00:00").unwrap();
        assert!(rfc.contains('T'));
        assert!(rfc.ends_with('Z'));
        assert_eq!(&rfc[0..10], "2026-08-17");
    }

    #[test]
    fn test_duration_one_hour() {
        let duration = duration_secs("2026-08-17 10:00:00", "2026-08-17 11:00:00").unwrap();
        assert_eq!(duration, 3600);
    }

    #[test]
    fn test_build_time_entry_body_includes_required_fields() {
        let entry = NewTimeEntry {
            description: "Coding - notes".into(),
            start: "2026-08-17T10:00:00Z".into(),
            stop: "2026-08-17T11:00:00Z".into(),
            duration: 3600,
            project_id: Some(42),
        };
        let body = build_time_entry_body(99, &entry);
        assert_eq!(body["created_with"], "tempus");
        assert_eq!(body["description"], "Coding - notes");
        assert_eq!(body["start"], "2026-08-17T10:00:00Z");
        assert_eq!(body["stop"], "2026-08-17T11:00:00Z");
        assert_eq!(body["duration"], 3600);
        assert_eq!(body["workspace_id"], 99);
        assert_eq!(body["project_id"], 42);
    }

    #[test]
    fn test_build_time_entry_body_omits_project_when_missing() {
        let entry = NewTimeEntry {
            description: "Coding".into(),
            start: "2026-08-17T10:00:00Z".into(),
            stop: "2026-08-17T11:00:00Z".into(),
            duration: 3600,
            project_id: None,
        };
        let body = build_time_entry_body(99, &entry);
        assert!(body.get("project_id").is_none());
    }
}
