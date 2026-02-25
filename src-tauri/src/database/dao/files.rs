use crate::database::{Database, to_json_string};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i32,
    pub session_id: String,
    pub path: String,
    pub accessed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStat {
    pub path: String,
    pub count: i32,
    pub workspace: Option<String>,
}

impl FileRecord {
    pub fn new(session_id: String, path: String) -> Self {
        Self {
            id: 0,
            session_id,
            path,
            accessed_at: Utc::now(),
        }
    }
}

pub struct FileDao;

impl FileDao {
    pub async fn insert_file(
        db: &Arc<Database>,
        session_id: &str,
        path: &str,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO files (session_id, path) VALUES (?, ?)",
            (session_id, path),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_files_by_session(
        db: &Arc<Database>,
        session_id: &str,
    ) -> Result<Vec<FileRecord>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, path, accessed_at FROM files WHERE session_id = ? ORDER BY accessed_at ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let files = stmt
            .query_map([session_id], |row| {
                Ok(FileRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    path: row.get(2)?,
                    accessed_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<FileRecord>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(files)
    }

    pub async fn get_file_statistics(
        db: &Arc<Database>,
        workspace: Option<&str>,
    ) -> Result<Vec<FileStat>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let files = if let Some(ws) = workspace {
            // Get files with workspace information
            let mut stmt = conn
                .prepare(
                    "SELECT f.path, COUNT(*) as count, s.workspace
                     FROM files f
                     JOIN sessions s ON f.session_id = s.id
                     WHERE s.workspace = ?
                     GROUP BY f.path, s.workspace
                     ORDER BY count DESC",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([ws], |row| {
                Ok(FileStat {
                    path: row.get(0)?,
                    count: row.get(1)?,
                    workspace: Some(row.get(2)?),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<FileStat>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            // Get files without workspace restriction
            let mut stmt = conn
                .prepare(
                    "SELECT f.path, COUNT(*) as count, s.workspace
                     FROM files f
                     JOIN sessions s ON f.session_id = s.id
                     GROUP BY f.path, s.workspace
                     ORDER BY count DESC",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([], |row| {
                Ok(FileStat {
                    path: row.get(0)?,
                    count: row.get(1)?,
                    workspace: row.get(2)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<FileStat>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        };

        Ok(files)
    }

    pub async fn delete_files_by_session(
        db: &Arc<Database>,
        session_id: &str,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("DELETE FROM files WHERE session_id = ?", [session_id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}