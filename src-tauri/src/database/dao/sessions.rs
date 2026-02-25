use crate::database::{Database, to_json_string};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub workspace: Option<String>,
    pub prompt: Option<String>,
    pub start_time: DateTime<Utc>,
    pub file_count: i32,
    pub command_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    pub fn new(id: String, start_time: DateTime<Utc>) -> Self {
        Self {
            id,
            workspace: None,
            prompt: None,
            start_time,
            file_count: 0,
            command_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

pub struct SessionDao;

impl SessionDao {
    pub async fn upsert_session(
        db: &Arc<Database>,
        session_id: &str,
        workspace: Option<&str>,
        start_time: &DateTime<Utc>,
        prompt: Option<&str>,
        command_count: i32,
        file_count: i32,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "INSERT OR REPLACE INTO sessions
            (id, workspace, prompt, start_time, command_count, file_count, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
            (
                session_id,
                workspace,
                prompt,
                start_time.timestamp(),
                command_count,
                file_count,
            ),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_all_sessions(db: &Arc<Database>) -> Result<Vec<Session>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, workspace, prompt, start_time, file_count, command_count,
                created_at, updated_at FROM sessions ORDER BY start_time DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let sessions = stmt
            .query_map([], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    prompt: row.get(2)?,
                    start_time: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    file_count: row.get(4)?,
                    command_count: row.get(5)?,
                    created_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(6)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(7)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Session>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(sessions)
    }

    pub async fn get_session_by_id(db: &Arc<Database>, session_id: &str) -> Result<Option<Session>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, workspace, prompt, start_time, file_count, command_count,
                created_at, updated_at FROM sessions WHERE id = ?",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let session = stmt
            .query_row([session_id], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    prompt: row.get(2)?,
                    start_time: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    file_count: row.get(4)?,
                    command_count: row.get(5)?,
                    created_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(6)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(7)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(session)
    }

    pub async fn get_sessions_by_workspace(db: &Arc<Database>, workspace: &str) -> Result<Vec<Session>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, workspace, prompt, start_time, file_count, command_count,
                created_at, updated_at FROM sessions WHERE workspace = ? ORDER BY start_time DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let sessions = stmt
            .query_map([workspace], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    prompt: row.get(2)?,
                    start_time: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    file_count: row.get(4)?,
                    command_count: row.get(5)?,
                    created_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(6)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(7)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Session>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(sessions)
    }

    pub async fn delete_session(db: &Arc<Database>, session_id: &str) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("DELETE FROM sessions WHERE id = ?", [session_id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_active_sessions_count(db: &Arc<Database>) -> Result<i32, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count)
    }

    pub async fn get_session_timeline(db: &Arc<Database>, days: i32) -> Result<Vec<Session>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, workspace, prompt, start_time, file_count, command_count,
                created_at, updated_at FROM sessions
                WHERE start_time >= datetime('now', ?) ORDER BY start_time DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let sessions = stmt
            .query_map([format!("-{} days", days)], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    prompt: row.get(2)?,
                    start_time: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    file_count: row.get(4)?,
                    command_count: row.get(5)?,
                    created_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(6)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(7)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Session>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(sessions)
    }
}