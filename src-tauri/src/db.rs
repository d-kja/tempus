use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        std::fs::create_dir_all(&app_dir)?;
        let db_path = app_dir.join("hours.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL")?;
        Self::migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn migrate(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                toggl_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
            CREATE TABLE IF NOT EXISTS entries (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                description TEXT,
                start_time TEXT NOT NULL,
                end_time TEXT,
                project_id INTEGER REFERENCES projects(id) ON DELETE SET NULL,
                toggl_id INTEGER,
                created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT OR IGNORE INTO settings (key, value) VALUES ('always_on_top', 'true');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('window_x', '');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('window_y', '');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('export_path', '');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('toggl_api_token', '');
            INSERT OR IGNORE INTO settings (key, value) VALUES ('toggl_project', '');
            ",
        )?;

        add_column_if_missing(conn, "projects", "toggl_id", "INTEGER")?;
        add_column_if_missing(conn, "entries", "toggl_id", "INTEGER")?;

        Ok(())
    }
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    spec: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if column_exists(conn, table, column)? {
        return Ok(());
    }
    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column} {spec}"),
        [],
    )?;
    Ok(())
}

fn column_exists(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for name in names {
        if name? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        Database::migrate(&conn).unwrap();
        Database {
            conn: Mutex::new(conn),
        }
    }

    #[test]
    fn test_migration_creates_tables() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"entries".to_string()));
        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"settings".to_string()));
    }

    #[test]
    fn test_migration_adds_toggl_columns_and_token_setting() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        assert!(super::column_exists(&conn, "entries", "toggl_id").unwrap());
        assert!(super::column_exists(&conn, "projects", "toggl_id").unwrap());
        let token: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'toggl_api_token'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(token, "");
        let project: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'toggl_project'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(project, "");
    }

    #[test]
    fn test_default_settings() {
        let db = in_memory_db();
        let conn = db.conn.lock().unwrap();
        let val: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'always_on_top'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(val, "true");
    }
}
