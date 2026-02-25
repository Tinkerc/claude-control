use crate::database::{Database, to_json_string};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub id: i32,
    pub session_id: String,
    pub command: String,
    pub executed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStat {
    pub command: String,
    pub count: i32,
    pub workspace: Option<String>,
}

impl Command {
    pub fn new(session_id: String, command: String) -> Self {
        Self {
            id: 0,
            session_id,
            command,
            executed_at: Utc::now(),
        }
    }
}

pub struct CommandDao;

impl CommandDao {
    pub async fn insert_command(
        db: &Arc<Database>,
        session_id: &str,
        command: &str,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO commands (session_id, command) VALUES (?, ?)",
            (session_id, command),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_commands_by_session(
        db: &Arc<Database>,
        session_id: &str,
    ) -> Result<Vec<Command>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, command, executed_at FROM commands WHERE session_id = ? ORDER BY executed_at ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let commands = stmt
            .query_map([session_id], |row| {
                Ok(Command {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    command: row.get(2)?,
                    executed_at: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<Command>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(commands)
    }

    pub async fn get_command_statistics(
        db: &Arc<Database>,
        workspace: Option<&str>,
    ) -> Result<Vec<CommandStat>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let commands = if let Some(ws) = workspace {
            // Get commands with workspace information
            let mut stmt = conn
                .prepare(
                    "SELECT c.command, COUNT(*) as count, s.workspace
                     FROM commands c
                     JOIN sessions s ON c.session_id = s.id
                     WHERE s.workspace = ?
                     GROUP BY c.command, s.workspace
                     ORDER BY count DESC",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([ws], |row| {
                Ok(CommandStat {
                    command: row.get(0)?,
                    count: row.get(1)?,
                    workspace: Some(row.get(2)?),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<CommandStat>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            // Get commands without workspace restriction
            let mut stmt = conn
                .prepare(
                    "SELECT c.command, COUNT(*) as count, s.workspace
                     FROM commands c
                     JOIN sessions s ON c.session_id = s.id
                     GROUP BY c.command, s.workspace
                     ORDER BY count DESC",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;

            stmt.query_map([], |row| {
                Ok(CommandStat {
                    command: row.get(0)?,
                    count: row.get(1)?,
                    workspace: row.get(2)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<CommandStat>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?
        };

        Ok(commands)
    }

    pub async fn delete_commands_by_session(
        db: &Arc<Database>,
        session_id: &str,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute("DELETE FROM commands WHERE session_id = ?", [session_id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}