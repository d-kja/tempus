use crate::db::Database;
use rusqlite::params;
use std::fs;

pub fn export_markdown_impl(db: &Database, path: &str) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT e.start_time, e.end_time, e.title, e.description, p.name
             FROM entries e
             LEFT JOIN projects p ON p.id = e.project_id
             ORDER BY e.start_time ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut content = String::from("| Date | Hour Span | Title | Description | Project |\n");
    content.push_str("|------|-----------|-------|-------------|--------|\n");

    for row in rows {
        let (start, end, title, desc, project) = row.map_err(|e| e.to_string())?;
        let date = start.chars().take(10).collect::<String>();
        let hour_span = format_hour_span(&start, end.as_deref());
        let desc_str = desc.unwrap_or_default();
        let proj_str = project.unwrap_or_default();
        content.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            date, hour_span, title, desc_str, proj_str
        ));
    }

    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn format_hour_span(start: &str, end: Option<&str>) -> String {
    let start_time = &start[11..16];
    let end_time = end.map(|e| &e[11..16]).unwrap_or("...");
    format!("{} - {}", start_time, end_time)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::entries::{start_entry_impl, stop_entry_impl};
    use rusqlite::Connection;
    use std::fs;

    fn setup_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::Database::migrate(&conn).unwrap();
        Database {
            conn: std::sync::Mutex::new(conn),
        }
    }

    #[test]
    fn test_export_markdown() {
        let db = setup_db();
        start_entry_impl(&db, "Coding", None, None).unwrap();
        stop_entry_impl(&db).unwrap();

        let tmp = std::env::temp_dir().join("hours_export_test.md");
        let path = tmp.to_str().unwrap();
        export_markdown_impl(&db, path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("Coding"));
        assert!(content.contains("| Date | Hour Span |"));
        let _ = fs::remove_file(tmp);
    }

    #[test]
    fn test_export_with_project() {
        use crate::commands::projects::create_project_impl;

        let db = setup_db();
        let project = create_project_impl(&db, "MyProject").unwrap();
        start_entry_impl(&db, "Task with project", None, Some(project.id)).unwrap();
        stop_entry_impl(&db).unwrap();

        let tmp = std::env::temp_dir().join("hours_export_project_test.md");
        let path = tmp.to_str().unwrap();
        export_markdown_impl(&db, path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("MyProject"));
        let _ = fs::remove_file(tmp);
    }
}
