use rusqlite::{OptionalExtension, params};

use crate::db::Database;
use crate::models::{HistoryEntry, SearchResult, Session};

pub fn create(
    database: &Database,
    name: &str,
    language: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    database.connection().execute(
        "INSERT OR IGNORE INTO sessions (name, language) VALUES (?1, ?2)",
        params![name, language],
    )?;
    switch(database, name)?;
    Ok(())
}

pub fn list(database: &Database) -> Result<Vec<Session>, Box<dyn std::error::Error>> {
    let mut statement = database
        .connection()
        .prepare("SELECT id, name, language, created_at FROM sessions ORDER BY created_at DESC")?;
    let sessions = statement
        .query_map([], row_to_session)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(sessions)
}

pub fn delete(database: &Database, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(id) = session_id(database, name)? {
        database.connection().execute(
            "DELETE FROM session_queries WHERE session_id = ?1",
            params![id],
        )?;
        database
            .connection()
            .execute("DELETE FROM bookmarks WHERE session_id = ?1", params![id])?;
        database
            .connection()
            .execute("DELETE FROM pins WHERE session_id = ?1", params![id])?;
    }
    database
        .connection()
        .execute("DELETE FROM sessions WHERE name = ?1", params![name])?;
    Ok(())
}

pub fn record_query(
    database: &Database,
    session_name: &str,
    query: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    database.connection().execute(
        "
        INSERT INTO session_queries (session_id, query)
        SELECT id, ?2 FROM sessions WHERE name = ?1
        ",
        params![session_name, query],
    )?;
    Ok(())
}

pub fn switch(database: &Database, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    if session_id(database, name)?.is_none() {
        return Err(format!("unknown session '{name}'").into());
    }
    database.connection().execute(
        "INSERT OR REPLACE INTO app_state (key, value) VALUES ('active_session', ?1)",
        params![name],
    )?;
    Ok(())
}

pub fn active(database: &Database) -> Result<Option<Session>, Box<dyn std::error::Error>> {
    let name = database
        .connection()
        .query_row(
            "SELECT value FROM app_state WHERE key = 'active_session'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    match name {
        Some(name) => get(database, &name),
        None => Ok(None),
    }
}

pub fn ensure_default(
    database: &Database,
    language: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    if let Some(session) = active(database)? {
        return Ok(session);
    }
    create(database, "default", language)?;
    Ok(active(database)?.expect("default session should exist after creation"))
}

pub fn get(database: &Database, name: &str) -> Result<Option<Session>, Box<dyn std::error::Error>> {
    Ok(database
        .connection()
        .query_row(
            "SELECT id, name, language, created_at FROM sessions WHERE name = ?1",
            params![name],
            row_to_session,
        )
        .optional()?)
}

pub fn history(
    database: &Database,
    session_name: &str,
    limit: usize,
) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
    let mut statement = database.connection().prepare(
        "
        SELECT sq.query, sq.timestamp
        FROM session_queries sq
        JOIN sessions s ON s.id = sq.session_id
        WHERE s.name = ?1
        ORDER BY sq.timestamp DESC
        LIMIT ?2
        ",
    )?;
    Ok(statement
        .query_map(params![session_name, limit as i64], |row| {
            Ok(HistoryEntry {
                query: row.get(0)?,
                timestamp: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?)
}

pub fn bookmark(
    database: &Database,
    session_name: &str,
    result: &SearchResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = session_id(database, session_name)?.ok_or("active session disappeared")?;
    database.connection().execute(
        "INSERT OR IGNORE INTO bookmarks (session_id, language, topic) VALUES (?1, ?2, ?3)",
        params![id, result.language, result.topic],
    )?;
    Ok(())
}

pub fn pin(
    database: &Database,
    session_name: &str,
    result: &SearchResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let id = session_id(database, session_name)?.ok_or("active session disappeared")?;
    database.connection().execute(
        "INSERT OR IGNORE INTO pins (session_id, language, topic) VALUES (?1, ?2, ?3)",
        params![id, result.language, result.topic],
    )?;
    Ok(())
}

pub fn saved_topics(
    database: &Database,
    table: &str,
    session_name: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if table != "bookmarks" && table != "pins" {
        return Err("invalid saved-topic table".into());
    }
    let sql = format!(
        "
        SELECT saved.language || ':' || saved.topic
        FROM {table} saved
        JOIN sessions s ON s.id = saved.session_id
        WHERE s.name = ?1
        ORDER BY saved.created_at DESC
        "
    );
    let mut statement = database.connection().prepare(&sql)?;
    Ok(statement
        .query_map(params![session_name], |row| row.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?)
}

pub fn clear_saved_topics(
    database: &Database,
    table: &str,
    session_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if table != "bookmarks" && table != "pins" {
        return Err("invalid saved-topic table".into());
    }
    let id = session_id(database, session_name)?.ok_or("active session disappeared")?;
    let sql = format!("DELETE FROM {table} WHERE session_id = ?1");
    database.connection().execute(&sql, params![id])?;
    Ok(())
}

pub fn stats(
    database: &Database,
    session_name: &str,
) -> Result<Vec<(String, i64)>, Box<dyn std::error::Error>> {
    let mut statement = database.connection().prepare(
        "
        SELECT sq.query, COUNT(*) AS count
        FROM session_queries sq
        JOIN sessions s ON s.id = sq.session_id
        WHERE s.name = ?1
        GROUP BY sq.query
        ORDER BY count DESC, sq.query
        LIMIT 20
        ",
    )?;
    Ok(statement
        .query_map(params![session_name], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?)
}

fn session_id(database: &Database, name: &str) -> rusqlite::Result<Option<i64>> {
    database
        .connection()
        .query_row(
            "SELECT id FROM sessions WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )
        .optional()
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    Ok(Session {
        id: row.get(0)?,
        name: row.get(1)?,
        language: row.get(2)?,
        created_at: row.get(3)?,
    })
}
