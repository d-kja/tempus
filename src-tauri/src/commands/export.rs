use crate::db::Database;
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

    let mut total_seconds: i64 = 0;
    let mut completed_count = 0;

    for row in rows {
        let (start, end, title, desc, project) = row.map_err(|e| e.to_string())?;
        let date = start.chars().take(10).collect::<String>();
        let hour_span = format_hour_span(&start, end.as_deref());
        let desc_str = escape_markdown_cell(&desc.unwrap_or_default());
        let proj_str = escape_markdown_cell(&project.unwrap_or_default());
        let title = escape_markdown_cell(&title);
        content.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            date, hour_span, title, desc_str, proj_str
        ));

        if let Some(ref end_time) = end {
            if let Ok(duration) = compute_duration_seconds(&start, end_time) {
                total_seconds += duration;
                completed_count += 1;
            }
        }
    }

    if completed_count > 0 {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        content.push_str(&format!("\n**Total:** {}h {}m\n", hours, minutes));
    }

    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

fn parse_to_seconds(s: &str) -> Result<i64, String> {
    if s.len() < 19 {
        return Err("invalid timestamp format".into());
    }
    let year: i64 = s[0..4].parse().map_err(|_| "invalid year")?;
    let month: i64 = s[5..7].parse().map_err(|_| "invalid month")?;
    let day: i64 = s[8..10].parse().map_err(|_| "invalid day")?;
    let hour: i64 = s[11..13].parse().map_err(|_| "invalid hour")?;
    let minute: i64 = s[14..16].parse().map_err(|_| "invalid minute")?;
    let second: i64 = s[17..19].parse().map_err(|_| "invalid second")?;

    let days = {
        let m = month;
        let y = if m <= 2 { year - 1 } else { year };
        let m = if m <= 2 { m + 12 } else { m };
        let a = y / 100;
        let b = 2 - a + a / 4;
        (36525 * y) / 100 + (306001 * (m + 1)) / 10000 + day + b - 694065
    };
    Ok(days * 86400 + hour * 3600 + minute * 60 + second)
}

fn compute_duration_seconds(start: &str, end: &str) -> Result<i64, String> {
    let start_secs = parse_to_seconds(start)?;
    let end_secs = parse_to_seconds(end)?;
    Ok(end_secs - start_secs)
}

fn escape_markdown_cell(value: &str) -> String {
    value.replace('|', "\\|").replace(['\n', '\r'], " ")
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
        assert!(content.contains("**Total:**"));
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

    #[test]
    fn test_export_includes_description() {
        let db = setup_db();
        start_entry_impl(
            &db,
            "Documented task",
            Some("Prepare the handoff notes"),
            None,
        )
        .unwrap();
        stop_entry_impl(&db).unwrap();

        let tmp = std::env::temp_dir().join("hours_export_description_test.md");
        let path = tmp.to_str().unwrap();
        export_markdown_impl(&db, path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("Prepare the handoff notes"));
        let _ = fs::remove_file(tmp);
    }
}
