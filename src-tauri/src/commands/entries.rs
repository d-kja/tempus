use crate::db::Database;
use crate::models::Entry;
use rusqlite::params;

pub fn start_entry_impl(
    db: &Database,
    title: &str,
    description: Option<&str>,
    project_id: Option<i64>,
) -> Result<Entry, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let title = if title.trim().is_empty() || title.eq_ignore_ascii_case("undefined") {
        "Untitled"
    } else {
        title
    };

    conn.execute(
        "UPDATE entries SET end_time = datetime('now'), updated_at = datetime('now') WHERE end_time IS NULL",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO entries (title, description, start_time, project_id) VALUES (?1, ?2, datetime('now'), ?3)",
        params![title, description, project_id],
    )
    .map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
        params![id],
        |row| Entry::from_row(row),
    )
    .map_err(|e| e.to_string())
}

pub fn stop_entry_impl(db: &Database) -> Result<Option<Entry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let active: Option<i64> = conn
        .query_row("SELECT id FROM entries WHERE end_time IS NULL", [], |row| {
            row.get(0)
        })
        .ok();

    if let Some(active_id) = active {
        conn.execute(
            "UPDATE entries SET end_time = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
            params![active_id],
        )
        .map_err(|e| e.to_string())?;

        let entry = conn
            .query_row(
                "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
                params![active_id],
                |row| Entry::from_row(row),
            )
            .map_err(|e| e.to_string())?;
        Ok(Some(entry))
    } else {
        Ok(None)
    }
}

pub fn get_active_entry_impl(db: &Database) -> Result<Option<Entry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE end_time IS NULL")
        .map_err(|e| e.to_string())?;

    let result = stmt
        .query_map([], |row| Entry::from_row(row))
        .map_err(|e| e.to_string())?
        .next()
        .transpose()
        .map_err(|e| e.to_string())?;

    Ok(result)
}

pub fn get_entries_impl(db: &Database, limit: i64, offset: i64) -> Result<Vec<Entry>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries ORDER BY start_time DESC LIMIT ?1 OFFSET ?2")
        .map_err(|e| e.to_string())?;

    let entries = stmt
        .query_map(params![limit, offset], |row| Entry::from_row(row))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(entries)
}

pub fn update_entry_impl(
    db: &Database,
    id: i64,
    title: &str,
    description: Option<&str>,
    project_id: Option<i64>,
    start_time: &str,
    end_time: Option<&str>,
) -> Result<Entry, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let _current: Entry = conn
        .query_row(
            "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
            params![id],
            |row| Entry::from_row(row),
        )
        .map_err(|e| format!("Entry not found: {}", e))?;

    conn.execute(
        "UPDATE entries SET title = ?1, description = ?2, project_id = ?3, start_time = ?4, end_time = ?5, updated_at = datetime('now') WHERE id = ?6",
        params![title, description, project_id, start_time, end_time, id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
        params![id],
        |row| Entry::from_row(row),
    )
    .map_err(|e| e.to_string())
}

pub fn resume_entry_impl(db: &Database, id: i64) -> Result<Entry, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let source: Entry = conn
        .query_row(
            "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
            params![id],
            |row| Entry::from_row(row),
        )
        .map_err(|e| format!("Entry not found: {}", e))?;

    conn.execute(
        "UPDATE entries SET end_time = datetime('now'), updated_at = datetime('now') WHERE end_time IS NULL",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO entries (title, description, start_time, project_id) VALUES (?1, ?2, datetime('now'), ?3)",
        params![source.title, source.description, source.project_id],
    ).map_err(|e| e.to_string())?;

    let new_id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
        params![new_id],
        |row| Entry::from_row(row),
    ).map_err(|e| e.to_string())
}

pub fn delete_entry_impl(db: &Database, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_all_entries_impl(db: &Database) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM entries", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    use rusqlite::Connection;

    fn setup_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::Database::migrate(&conn).unwrap();
        Database {
            conn: std::sync::Mutex::new(conn),
        }
    }

    #[test]
    fn test_start_entry_creates_active_entry() {
        let db = setup_db();
        let entry = start_entry_impl(&db, "Test Task", None, None).unwrap();
        assert_eq!(entry.title, "Test Task");
        assert!(entry.end_time.is_none());
        assert!(!entry.start_time.is_empty());
    }

    #[test]
    fn test_start_entry_defaults_undefined_title() {
        let db = setup_db();
        let entry = start_entry_impl(&db, "Undefined", None, None).unwrap();
        assert_eq!(entry.title, "Untitled");
    }

    #[test]
    fn test_start_entry_auto_stops_previous() {
        let db = setup_db();
        let first = start_entry_impl(&db, "First", None, None).unwrap();
        let second = start_entry_impl(&db, "Second", None, None).unwrap();

        let conn = db.conn.lock().unwrap();
        let end_time: String = conn
            .query_row(
                "SELECT end_time FROM entries WHERE id = ?1",
                rusqlite::params![first.id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!end_time.is_empty());
    }

    #[test]
    fn test_start_entry_with_project_id() {
        use crate::commands::projects::create_project_impl;

        let db = setup_db();
        let project = create_project_impl(&db, "TestProj").unwrap();
        let entry = start_entry_impl(&db, "Task with project", None, Some(project.id)).unwrap();
        assert_eq!(entry.project_id, Some(project.id));

        let active = get_active_entry_impl(&db).unwrap().unwrap();
        assert_eq!(active.project_id, Some(project.id));
        assert_eq!(active.title, "Task with project");
    }

    #[test]
    fn test_start_entry_without_project_id() {
        let db = setup_db();
        let entry = start_entry_impl(&db, "Task without project", None, None).unwrap();
        assert!(entry.project_id.is_none());
    }

    #[test]
    fn test_stop_entry_sets_end_time() {
        let db = setup_db();
        start_entry_impl(&db, "Task", None, None).unwrap();
        let stopped = stop_entry_impl(&db).unwrap().unwrap();
        assert!(stopped.end_time.is_some());
    }

    #[test]
    fn test_resume_creates_new_entry_with_same_information() {
        let db = setup_db();
        let project = crate::commands::projects::create_project_impl(&db, "Project").unwrap();
        let source = start_entry_impl(&db, "Task", Some("Details"), Some(project.id)).unwrap();
        let stopped = stop_entry_impl(&db).unwrap().unwrap();
        let resumed = resume_entry_impl(&db, source.id).unwrap();

        assert_eq!(stopped.id, source.id);
        assert!(stopped.end_time.is_some());
        assert_ne!(resumed.id, source.id);
        assert_eq!(resumed.title, source.title);
        assert_eq!(resumed.description, source.description);
        assert_eq!(resumed.project_id, source.project_id);
        assert!(resumed.end_time.is_none());
    }

    #[test]
    fn test_get_active_entry_returns_none_when_idle() {
        let db = setup_db();
        assert!(get_active_entry_impl(&db).unwrap().is_none());
    }

    #[test]
    fn test_get_entries_returns_all() {
        let db = setup_db();
        start_entry_impl(&db, "A", None, None).unwrap();
        stop_entry_impl(&db).unwrap();
        start_entry_impl(&db, "B", None, None).unwrap();
        let entries = get_entries_impl(&db, 10, 0).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_delete_entry_removes_it() {
        let db = setup_db();
        let entry = start_entry_impl(&db, "Del", None, None).unwrap();
        delete_entry_impl(&db, entry.id).unwrap();
        let entries = get_entries_impl(&db, 10, 0).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_update_entry_updates_details_and_times() {
        let db = setup_db();
        let entry = start_entry_impl(&db, "Original", Some("Notes"), None).unwrap();
        let updated = update_entry_impl(
            &db,
            entry.id,
            "Updated",
            Some("Updated notes"),
            None,
            "2026-07-12 09:14:00",
            Some("2026-07-12 10:40:00"),
        )
        .unwrap();

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.description.as_deref(), Some("Updated notes"));
        assert_eq!(updated.start_time, "2026-07-12 09:14:00");
        assert_eq!(updated.end_time.as_deref(), Some("2026-07-12 10:40:00"));
    }
}
