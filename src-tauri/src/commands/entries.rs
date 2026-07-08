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
        .query_row(
            "SELECT id FROM entries WHERE end_time IS NULL",
            [],
            |row| row.get(0),
        )
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
    title: Option<&str>,
    description: Option<Option<&str>>,
    project_id: Option<Option<i64>>,
) -> Result<Entry, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let current: Entry = conn
        .query_row(
            "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
            params![id],
            |row| Entry::from_row(row),
        )
        .map_err(|e| format!("Entry not found: {}", e))?;

    let new_title = title.unwrap_or(&current.title).to_string();
    let new_desc = match description {
        Some(d) => d.map(|s| s.to_string()),
        None => current.description.clone(),
    };
    let new_project = match project_id {
        Some(p) => p,
        None => current.project_id,
    };

    conn.execute(
        "UPDATE entries SET title = ?1, description = ?2, project_id = ?3, updated_at = datetime('now') WHERE id = ?4",
        params![new_title, new_desc, new_project, id],
    )
    .map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT id, title, description, start_time, end_time, project_id, created_at, updated_at FROM entries WHERE id = ?1",
        params![id],
        |row| Entry::from_row(row),
    )
    .map_err(|e| e.to_string())
}

pub fn delete_entry_impl(db: &Database, id: i64) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])
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
    fn test_stop_entry_sets_end_time() {
        let db = setup_db();
        start_entry_impl(&db, "Task", None, None).unwrap();
        let stopped = stop_entry_impl(&db).unwrap().unwrap();
        assert!(stopped.end_time.is_some());
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
}
