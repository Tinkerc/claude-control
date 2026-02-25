use crate::database::{Database, dao};
use crate::error::AppError;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub workspace: Option<String>,
    pub confidence: f64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct MemoryService {
    db: Arc<Database>,
}

impl MemoryService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn extract_memory_from_session(&self, session_id: &str) -> Result<(), AppError> {
        log::debug!("Extracting memory from session: {}", session_id);

        // Get session data
        let session = match dao::SessionDao::get_session_by_id(&self.db, session_id).await? {
            Some(session) => session,
            None => {
                log::warn!("Session not found: {}", session_id);
                return Ok(());
            }
        };

        // Get associated commands for this session
        let commands = dao::CommandDao::get_commands_by_session(&self.db, session_id).await?;

        // Extract memory based on command patterns
        let mut command_frequency: HashMap<String, u32> = HashMap::new();

        for command in &commands {
            *command_frequency.entry(command.command.clone()).or_insert(0) += 1;
        }

        // Identify frequent commands (potential memory candidates)
        for (command, count) in command_frequency {
            if count >= 2 { // Threshold for considering a command as frequently used
                let confidence = (count as f64 / commands.len() as f64).min(1.0);

                // Determine workspace for this memory
                let workspace = session.workspace.clone();

                // Store memory entry
                self.store_memory_entry(&command, &command, workspace, confidence).await?;
            }
        }

        Ok(())
    }

    pub async fn store_memory_entry(&self, key: &str, value: &str, workspace: Option<String>, confidence: f64) -> Result<(), AppError> {
        dao::MemoryDao::upsert_memory(&self.db, key, value, workspace, confidence).await
    }

    pub async fn get_memory_entries(&self, workspace: Option<&str>) -> Result<Vec<MemoryEntry>, AppError> {
        let entries = dao::MemoryDao::get_memories(&self.db, workspace).await?;

        // Convert from database model to service model
        let memory_entries = entries.into_iter()
            .map(|db_entry| MemoryEntry {
                key: db_entry.key,
                value: db_entry.value,
                workspace: db_entry.workspace,
                confidence: db_entry.confidence,
                updated_at: db_entry.updated_at,
            })
            .collect();

        Ok(memory_entries)
    }

    pub async fn get_memory_entry(&self, key: &str, workspace: Option<&str>) -> Result<Option<MemoryEntry>, AppError> {
        if let Some(db_entry) = dao::MemoryDao::get_memory_by_key(&self.db, key, workspace).await? {
            Ok(Some(MemoryEntry {
                key: db_entry.key,
                value: db_entry.value,
                workspace: db_entry.workspace,
                confidence: db_entry.confidence,
                updated_at: db_entry.updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_memory_entry(&self, key: &str, value: &str, workspace: Option<String>, confidence: f64) -> Result<(), AppError> {
        self.store_memory_entry(key, value, workspace, confidence).await
    }

    pub async fn delete_memory_entry(&self, key: &str, workspace: Option<&str>) -> Result<(), AppError> {
        dao::MemoryDao::delete_memory(&self.db, key, workspace).await
    }

    pub async fn get_top_memories(&self, workspace: Option<&str>, limit: usize) -> Result<Vec<MemoryEntry>, AppError> {
        let entries = dao::MemoryDao::get_top_memories(&self.db, workspace, limit).await?;

        // Convert from database model to service model
        let memory_entries = entries.into_iter()
            .map(|db_entry| MemoryEntry {
                key: db_entry.key,
                value: db_entry.value,
                workspace: db_entry.workspace,
                confidence: db_entry.confidence,
                updated_at: db_entry.updated_at,
            })
            .collect();

        Ok(memory_entries)
    }
}