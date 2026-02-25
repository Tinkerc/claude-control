use crate::database::{Database, dao};
use crate::error::AppError;
use std::sync::Arc;

pub struct InsightService {
    db: Arc<Database>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionStat {
    pub session_id: String,
    pub workspace: Option<String>,
    pub prompt: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub file_count: i32,
    pub command_count: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandStat {
    pub command: String,
    pub count: i32,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileStat {
    pub file_path: String,
    pub count: i32,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptStat {
    pub prompt: String,
    pub count: i32,
}

impl InsightService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_session_stats(&self) -> Result<Vec<SessionStat>, AppError> {
        let sessions = dao::SessionDao::get_all_sessions(&self.db).await?;

        let stats = sessions.into_iter()
            .map(|session| SessionStat {
                session_id: session.id,
                workspace: session.workspace,
                prompt: session.prompt,
                start_time: session.start_time,
                file_count: session.file_count,
                command_count: session.command_count,
            })
            .collect();

        Ok(stats)
    }

    pub async fn get_sessions_by_workspace(&self, workspace: &str) -> Result<Vec<SessionStat>, AppError> {
        let sessions = dao::SessionDao::get_sessions_by_workspace(&self.db, workspace).await?;

        let stats = sessions.into_iter()
            .map(|session| SessionStat {
                session_id: session.id,
                workspace: session.workspace,
                prompt: session.prompt,
                start_time: session.start_time,
                file_count: session.file_count,
                command_count: session.command_count,
            })
            .collect();

        Ok(stats)
    }

    pub async fn get_command_statistics(&self, workspace: Option<&str>) -> Result<Vec<CommandStat>, AppError> {
        let commands = dao::CommandDao::get_command_statistics(&self.db, workspace).await?;

        let stats = commands.into_iter()
            .map(|command| CommandStat {
                command: command.command,
                count: command.count,
                workspace: command.workspace,
            })
            .collect();

        Ok(stats)
    }

    pub async fn get_file_statistics(&self, workspace: Option<&str>) -> Result<Vec<FileStat>, AppError> {
        let files = dao::FileDao::get_file_statistics(&self.db, workspace).await?;

        let stats = files.into_iter()
            .map(|file| FileStat {
                file_path: file.path,
                count: file.count,
                workspace: file.workspace,
            })
            .collect();

        Ok(stats)
    }

    pub async fn get_prompt_statistics(&self) -> Result<Vec<PromptStat>, AppError> {
        let prompts = dao::PromptDao::get_prompt_statistics(&self.db).await?;

        let stats = prompts.into_iter()
            .map(|prompt| PromptStat {
                prompt: prompt.text,
                count: prompt.count,
            })
            .collect();

        Ok(stats)
    }

    pub async fn get_workspace_activity(&self) -> Result<Vec<(String, i32)>, AppError> {
        // Get the count of sessions per workspace
        let sessions = dao::SessionDao::get_all_sessions(&self.db).await?;

        let mut workspace_counts: std::collections::HashMap<String, i32> = std::collections::HashMap::new();

        for session in sessions {
            if let Some(workspace) = session.workspace {
                *workspace_counts.entry(workspace).or_insert(0) += 1;
            }
        }

        let mut result: Vec<(String, i32)> = workspace_counts.into_iter().collect();
        // Sort by count in descending order
        result.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(result)
    }

    pub async fn get_active_sessions_count(&self) -> Result<i32, AppError> {
        dao::SessionDao::get_active_sessions_count(&self.db).await
    }

    pub async fn get_session_timeline(&self, days: i32) -> Result<Vec<SessionStat>, AppError> {
        let sessions = dao::SessionDao::get_session_timeline(&self.db, days).await?;

        let stats = sessions.into_iter()
            .map(|session| SessionStat {
                session_id: session.id,
                workspace: session.workspace,
                prompt: session.prompt,
                start_time: session.start_time,
                file_count: session.file_count,
                command_count: session.command_count,
            })
            .collect();

        Ok(stats)
    }
}