use crate::db::Database;
use rusqlite::params;
use std::collections::HashMap;

pub fn get_settings_impl(db: &Database) -> Result<HashMap<String, String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings ORDER BY key")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?;

    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(|e| e.to_string())?;
        map.insert(k, v);
    }
    Ok(map)
}

pub fn update_settings_impl(db: &Database, settings: &HashMap<String, String>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    for (key, value) in settings {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::Database::migrate(&conn).unwrap();
        Database {
            conn: std::sync::Mutex::new(conn),
        }
    }

    #[test]
    fn test_get_default_settings() {
        let db = setup_db();
        let s = get_settings_impl(&db).unwrap();
        assert_eq!(s.get("always_on_top").unwrap(), "true");
    }

    #[test]
    fn test_update_settings() {
        let db = setup_db();
        let mut m = HashMap::new();
        m.insert("always_on_top".into(), "false".into());
        update_settings_impl(&db, &m).unwrap();
        let s = get_settings_impl(&db).unwrap();
        assert_eq!(s.get("always_on_top").unwrap(), "false");
    }
}
