use crate::database::{Database, to_json_string};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i32,
    pub workspace: Option<String>,
    pub key: String,
    pub value: String,
    pub confidence: f64,
    pub updated_at: DateTime<Utc>,
}

impl Memory {
    pub fn new(workspace: Option<String>, key: String, value: String, confidence: f64) -> Self {
        Self {
            id: 0,
            workspace,
            key,
            value,
            confidence,
            updated_at: Utc::now(),
        }
    }
}

pub struct MemoryDao;

impl MemoryDao {
    pub async fn upsert_memory(
        db: &Arc<Database>,
        key: &str,
        value: &str,
        workspace: Option<String>,
        confidence: f64,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "INSERT OR REPLACE INTO memory (workspace, key, value, confidence, updated_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
            (workspace, key, value, confidence),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_memories(
        db: &Arc<Database>,
        workspace: Option<&str>,
    ) -> Result<Vec<Memory>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let memories = if let Some(ws) = workspace {
            // Get memories for a specific workspace
            let mut stmt = conn
                .prepare(
                    "SELECT id, workspace, key, value, confidence, updated_at
                     FROM memory
                     WHERE workspace = ? OR workspace IS NULL
                     ORDER BY confidence DESC, updated_at DESC",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([ws], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    confidence: row.get(4)?,
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(5)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Memory>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            // Get all memories regardless of workspace
            let mut stmt = conn
                .prepare(
                    "SELECT id, workspace, key, value, confidence, updated_at
                     FROM memory
                     ORDER BY confidence DESC, updated_at DESC",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    confidence: row.get(4)?,
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(5)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Memory>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        };

        Ok(memories)
    }

    pub async fn get_memory_by_key(
        db: &Arc<Database>,
        key: &str,
        workspace: Option<&str>,
    ) -> Result<Option<Memory>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        // First try to find memory for the specific workspace
        if let Some(ws) = workspace {
            let mut stmt = conn
                .prepare(
                    "SELECT id, workspace, key, value, confidence, updated_at
                     FROM memory
                     WHERE key = ? AND workspace = ?
                     ORDER BY updated_at DESC LIMIT 1",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            if let Some(memory) = stmt
                .query_row([key, ws], |row| {
                    Ok(Memory {
                        id: row.get(0)?,
                        workspace: row.get(1)?,
                        key: row.get(2)?,
                        value: row.get(3)?,
                        confidence: row.get(4)?,
                        updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(5)?, 0)
                            .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                            .into(),
                    })
                })
                .optional()
                .map_err(|e| AppError::Database(e.to_string()))?
            {
                return Ok(Some(memory));
            }
        }

        // If not found in workspace-specific search or no workspace provided,
        // look for global memory (workspace = NULL)
        let mut stmt = conn
            .prepare(
                "SELECT id, workspace, key, value, confidence, updated_at
                 FROM memory
                 WHERE key = ? AND workspace IS NULL
                 ORDER BY updated_at DESC LIMIT 1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let memory = stmt
            .query_row([key], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    confidence: row.get(4)?,
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(5)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .optional()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(memory)
    }

    pub async fn delete_memory(
        db: &Arc<Database>,
        key: &str,
        workspace: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(ws) = workspace {
            conn.execute(
                "DELETE FROM memory WHERE key = ? AND workspace = ?",
                [key, ws],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        } else {
            conn.execute(
                "DELETE FROM memory WHERE key = ? AND workspace IS NULL",
                [key],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        Ok(())
    }

    pub async fn get_top_memories(
        db: &Arc<Database>,
        workspace: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Memory>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let memories = if let Some(ws) = workspace {
            // Get top memories for a specific workspace
            let mut stmt = conn
                .prepare(
                    "SELECT id, workspace, key, value, confidence, updated_at
                     FROM memory
                     WHERE workspace = ? OR workspace IS NULL
                     ORDER BY confidence DESC, updated_at DESC
                     LIMIT ?",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([ws, &(limit as i32)], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    confidence: row.get(4)?,
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(5)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Memory>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            // Get top memories regardless of workspace
            let mut stmt = conn
                .prepare(
                    "SELECT id, workspace, key, value, confidence, updated_at
                     FROM memory
                     ORDER BY confidence DESC, updated_at DESC
                     LIMIT ?",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([&limit as &dyn rusqlite::ToSql], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    workspace: row.get(1)?,
                    key: row.get(2)?,
                    value: row.get(3)?,
                    confidence: row.get(4)?,
                    updated_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(5)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Memory>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        };

        Ok(memories)
    }
}