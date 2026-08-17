use crate::commands::settings::get_settings_impl;
use crate::db::Database;
use crate::toggl::{
    duration_secs, sql_datetime_to_rfc3339, toggl_description, NewTimeEntry, TogglApi, TogglClient,
    TogglProject,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TogglSyncResult {
    pub created: u32,
    pub skipped: u32,
    pub projects_created: u32,
}

struct LocalEntry {
    id: i64,
    title: String,
    description: Option<String>,
    start_time: String,
    end_time: String,
    project_id: Option<i64>,
}

struct LocalProject {
    id: i64,
    name: String,
    toggl_id: Option<i64>,
}

pub fn sync_toggl_impl(db: &Database) -> Result<TogglSyncResult, String> {
    let settings = get_settings_impl(db)?;
    let token = settings
        .get("toggl_api_token")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if token.is_empty() {
        return Err("Add your Toggl API token in Settings first.".into());
    }

    let client = TogglClient::new(token)?;
    sync_with_api(db, &client)
}

pub fn sync_with_api(db: &Database, api: &dyn TogglApi) -> Result<TogglSyncResult, String> {
    let user = api.get_me()?;
    let workspace_id = user.default_workspace_id;
    let mut remote_projects = api.list_projects(workspace_id)?;
    let mut local_projects = load_projects(db)?;
    let unsynced = load_unsynced_entries(db)?;

    let mut created = 0;
    let mut skipped = 0;
    let mut projects_created = 0;

    for entry in unsynced {
        let duration = match duration_secs(&entry.start_time, &entry.end_time) {
            Ok(seconds) if seconds > 0 => seconds,
            _ => {
                skipped += 1;
                continue;
            }
        };

        let project_id = resolve_project_id(
            db,
            api,
            workspace_id,
            entry.project_id,
            &mut local_projects,
            &mut remote_projects,
            &mut projects_created,
        )?;

        let payload = NewTimeEntry {
            description: toggl_description(&entry.title, entry.description.as_deref()),
            start: sql_datetime_to_rfc3339(&entry.start_time)?,
            stop: sql_datetime_to_rfc3339(&entry.end_time)?,
            duration,
            project_id,
        };
        let remote = api.create_time_entry(workspace_id, &payload)?;
        save_entry_toggl_id(db, entry.id, remote.id)?;
        created += 1;
    }

    Ok(TogglSyncResult {
        created,
        skipped,
        projects_created,
    })
}

fn resolve_project_id(
    db: &Database,
    api: &dyn TogglApi,
    workspace_id: i64,
    local_project_id: Option<i64>,
    local_projects: &mut [LocalProject],
    remote_projects: &mut Vec<TogglProject>,
    projects_created: &mut u32,
) -> Result<Option<i64>, String> {
    let Some(local_project_id) = local_project_id else {
        return Ok(None);
    };
    let Some(index) = local_projects
        .iter()
        .position(|project| project.id == local_project_id)
    else {
        return Ok(None);
    };

    if let Some(toggl_id) = local_projects[index].toggl_id {
        return Ok(Some(toggl_id));
    }

    let name = local_projects[index].name.clone();
    if let Some(existing) = remote_projects
        .iter()
        .find(|project| project.name.eq_ignore_ascii_case(&name))
    {
        let toggl_id = existing.id;
        save_project_toggl_id(db, local_project_id, toggl_id)?;
        local_projects[index].toggl_id = Some(toggl_id);
        return Ok(Some(toggl_id));
    }

    let created = api.create_project(workspace_id, &name)?;
    save_project_toggl_id(db, local_project_id, created.id)?;
    local_projects[index].toggl_id = Some(created.id);
    remote_projects.push(created.clone());
    *projects_created += 1;
    Ok(Some(created.id))
}

fn load_projects(db: &Database) -> Result<Vec<LocalProject>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, toggl_id FROM projects")
        .map_err(|e| e.to_string())?;
    let projects = stmt
        .query_map([], |row| {
            Ok(LocalProject {
                id: row.get(0)?,
                name: row.get(1)?,
                toggl_id: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(projects)
}

fn load_unsynced_entries(db: &Database) -> Result<Vec<LocalEntry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, title, description, start_time, end_time, project_id
             FROM entries
             WHERE end_time IS NOT NULL AND toggl_id IS NULL
             ORDER BY start_time ASC",
        )
        .map_err(|e| e.to_string())?;
    let entries = stmt
        .query_map([], |row| {
            Ok(LocalEntry {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                project_id: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(entries)
}

fn save_entry_toggl_id(db: &Database, entry_id: i64, toggl_id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE entries SET toggl_id = ?1, updated_at = datetime('now', 'localtime') WHERE id = ?2",
        params![toggl_id, entry_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn save_project_toggl_id(db: &Database, project_id: i64, toggl_id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE projects SET toggl_id = ?1 WHERE id = ?2",
        params![toggl_id, project_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::entries::{start_entry_impl, stop_entry_impl, update_entry_impl};
    use crate::commands::projects::create_project_impl;
    use crate::toggl::{TogglTimeEntry, TogglUser};
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn setup_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::Database::migrate(&conn).unwrap();
        Database {
            conn: Mutex::new(conn),
        }
    }

    struct MockToggl {
        workspace_id: i64,
        projects: Mutex<Vec<TogglProject>>,
        entries: Mutex<Vec<NewTimeEntry>>,
        next_id: Mutex<i64>,
        fail: Mutex<Option<String>>,
    }

    impl MockToggl {
        fn new() -> Self {
            Self {
                workspace_id: 7,
                projects: Mutex::new(Vec::new()),
                entries: Mutex::new(Vec::new()),
                next_id: Mutex::new(100),
                fail: Mutex::new(None),
            }
        }

        fn with_projects(projects: Vec<TogglProject>) -> Self {
            let mock = Self::new();
            *mock.projects.lock().unwrap() = projects;
            mock
        }

        fn next_id(&self) -> i64 {
            let mut next = self.next_id.lock().unwrap();
            let id = *next;
            *next += 1;
            id
        }
    }

    fn complete_entry(
        db: &Database,
        title: &str,
        description: Option<&str>,
        project_id: Option<i64>,
        start: &str,
        end: &str,
    ) {
        let entry = start_entry_impl(db, title, description, project_id).unwrap();
        stop_entry_impl(db).unwrap();
        update_entry_impl(
            db,
            entry.id,
            title,
            description,
            project_id,
            start,
            Some(end),
        )
        .unwrap();
    }

    impl TogglApi for MockToggl {
        fn get_me(&self) -> Result<TogglUser, String> {
            if let Some(error) = self.fail.lock().unwrap().clone() {
                return Err(error);
            }
            Ok(TogglUser {
                default_workspace_id: self.workspace_id,
            })
        }

        fn list_projects(&self, _workspace_id: i64) -> Result<Vec<TogglProject>, String> {
            Ok(self.projects.lock().unwrap().clone())
        }

        fn create_project(&self, _workspace_id: i64, name: &str) -> Result<TogglProject, String> {
            let project = TogglProject {
                id: self.next_id(),
                name: name.to_string(),
            };
            self.projects.lock().unwrap().push(project.clone());
            Ok(project)
        }

        fn create_time_entry(
            &self,
            _workspace_id: i64,
            entry: &NewTimeEntry,
        ) -> Result<TogglTimeEntry, String> {
            self.entries.lock().unwrap().push(entry.clone());
            Ok(TogglTimeEntry { id: self.next_id() })
        }
    }

    #[test]
    fn test_sync_requires_api_token() {
        let db = setup_db();
        let error = sync_toggl_impl(&db).unwrap_err();
        assert!(error.contains("API token"));
    }

    #[test]
    fn test_sync_creates_completed_entry() {
        let db = setup_db();
        complete_entry(
            &db,
            "Coding",
            Some("Handoff notes"),
            None,
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );

        let api = MockToggl::new();
        let result = sync_with_api(&db, &api).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(result.skipped, 0);

        let created = api.entries.lock().unwrap();
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].description, "Coding - Handoff notes");
        assert!(created[0].start.contains('T'));
        assert!(created[0].stop.contains('T'));
        assert_eq!(created[0].duration, 3600);
        assert!(created[0].project_id.is_none());
    }

    #[test]
    fn test_sync_skips_running_and_already_synced_entries() {
        let db = setup_db();
        complete_entry(
            &db,
            "Done",
            None,
            None,
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );
        start_entry_impl(&db, "Still running", None, None).unwrap();

        let api = MockToggl::new();
        let first = sync_with_api(&db, &api).unwrap();
        assert_eq!(first.created, 1);

        let second = sync_with_api(&db, &api).unwrap();
        assert_eq!(second.created, 0);
        assert_eq!(api.entries.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_sync_matches_existing_toggl_project_by_name() {
        let db = setup_db();
        let project = create_project_impl(&db, "Tempus").unwrap();
        complete_entry(
            &db,
            "Build",
            None,
            Some(project.id),
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );

        let api = MockToggl::with_projects(vec![TogglProject {
            id: 55,
            name: "tempus".into(),
        }]);
        let result = sync_with_api(&db, &api).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(result.projects_created, 0);
        assert_eq!(api.entries.lock().unwrap()[0].project_id, Some(55));
        assert_eq!(api.projects.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_sync_creates_missing_toggl_project() {
        let db = setup_db();
        let project = create_project_impl(&db, "Hours").unwrap();
        complete_entry(
            &db,
            "Build",
            None,
            Some(project.id),
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );

        let api = MockToggl::new();
        let result = sync_with_api(&db, &api).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(result.projects_created, 1);

        let remote_projects = api.projects.lock().unwrap();
        assert_eq!(remote_projects.len(), 1);
        assert_eq!(remote_projects[0].name, "Hours");
        assert_eq!(
            api.entries.lock().unwrap()[0].project_id,
            Some(remote_projects[0].id)
        );
    }

    #[test]
    fn test_sync_reuses_cached_project_id_on_second_entry() {
        let db = setup_db();
        let project = create_project_impl(&db, "Hours").unwrap();
        complete_entry(
            &db,
            "One",
            None,
            Some(project.id),
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );
        complete_entry(
            &db,
            "Two",
            None,
            Some(project.id),
            "2026-08-17 12:00:00",
            "2026-08-17 13:00:00",
        );

        let api = MockToggl::new();
        let result = sync_with_api(&db, &api).unwrap();
        assert_eq!(result.created, 2);
        assert_eq!(result.projects_created, 1);
        assert_eq!(api.projects.lock().unwrap().len(), 1);

        let entries = api.entries.lock().unwrap();
        assert_eq!(entries[0].project_id, entries[1].project_id);
        assert!(entries[0].project_id.is_some());
    }

    #[test]
    fn test_sync_propagates_api_errors() {
        let db = setup_db();
        complete_entry(
            &db,
            "Coding",
            None,
            None,
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );

        let api = MockToggl::new();
        *api.fail.lock().unwrap() = Some("Invalid Toggl API token.".into());
        let error = sync_with_api(&db, &api).unwrap_err();
        assert_eq!(error, "Invalid Toggl API token.");
    }

    #[test]
    fn test_sync_persists_toggl_ids() {
        let db = setup_db();
        let project = create_project_impl(&db, "Hours").unwrap();
        complete_entry(
            &db,
            "Build",
            None,
            Some(project.id),
            "2026-08-17 10:00:00",
            "2026-08-17 11:00:00",
        );

        let api = MockToggl::new();
        sync_with_api(&db, &api).unwrap();

        let conn = db.conn.lock().unwrap();
        let entry_toggl_id: Option<i64> = conn
            .query_row("SELECT toggl_id FROM entries LIMIT 1", [], |row| row.get(0))
            .unwrap();
        let project_toggl_id: Option<i64> = conn
            .query_row("SELECT toggl_id FROM projects LIMIT 1", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(entry_toggl_id.is_some());
        assert!(project_toggl_id.is_some());
    }
}
