use crate::database::{Database, to_json_string};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptStat {
    pub id: i32,
    pub text: String,
    pub count: i32,
    pub last_used: DateTime<Utc>,
}

impl PromptStat {
    pub fn new(text: String) -> Self {
        Self {
            id: 0,
            text,
            count: 1,
            last_used: Utc::now(),
        }
    }
}

pub struct PromptDao;

impl PromptDao {
    pub async fn upsert_prompt_stat(
        db: &Arc<Database>,
        prompt_text: &str,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "INSERT INTO prompt_stats (text, count, last_used)
             VALUES (?, 1, datetime('now'))
             ON CONFLICT(text) DO UPDATE SET
             count = count + 1,
             last_used = datetime('now')",
            (prompt_text,),
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn get_prompt_statistics(
        db: &Arc<Database>,
    ) -> Result<Vec<PromptStat>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, text, count, last_used
                 FROM prompt_stats
                 ORDER BY count DESC, last_used DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let prompts = stmt
            .query_map([], |row| {
                Ok(PromptStat {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    count: row.get(2)?,
                    last_used: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<PromptStat>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(prompts)
    }

    pub async fn get_top_prompts(
        db: &Arc<Database>,
        limit: usize,
    ) -> Result<Vec<PromptStat>, AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, text, count, last_used
                 FROM prompt_stats
                 ORDER BY count DESC, last_used DESC
                 LIMIT ?",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let prompts = stmt
            .query_map([&limit as &dyn rusqlite::ToSql], |row| {
                Ok(PromptStat {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    count: row.get(2)?,
                    last_used: DateTime::<Utc>::from_timestamp(row.get::<_, i64>(3)?, 0)
                        .ok_or_else(|| rusqlite::Error::FromSqlConversionOverflow)?
                        .into(),
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<Result<Vec<PromptStat>, _>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(prompts)
    }

    pub async fn delete_prompt_stat(
        db: &Arc<Database>,
        prompt_text: &str,
    ) -> Result<(), AppError> {
        let conn = db.conn.lock().map_err(|e| AppError::Database(e.to_string()))?;

        conn.execute(
            "DELETE FROM prompt_stats WHERE text = ?",
            [prompt_text],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}