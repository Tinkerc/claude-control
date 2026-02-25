use crate::database::{Database, dao};
use crate::error::AppError;
use crate::watcher::ClaudeWatcherEvent;
use crate::parser::{SessionParser, ParsedSessionData};
use std::sync::Arc;
use std::path::PathBuf;

pub struct SessionService {
    db: Arc<Database>,
    parser: SessionParser,
}

impl SessionService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            parser: SessionParser::new(),
        }
    }

    pub async fn get_all_sessions(&self) -> Result<Vec<dao::sessions::Session>, AppError> {
        dao::sessions::SessionDao::get_all_sessions(&self.db).await
    }

    pub async fn get_sessions_by_workspace(&self, workspace: &str) -> Result<Vec<dao::sessions::Session>, AppError> {
        dao::sessions::SessionDao::get_sessions_by_workspace(&self.db, workspace).await
    }

    pub async fn get_session_by_id(&self, session_id: &str) -> Result<Option<dao::sessions::Session>, AppError> {
        dao::sessions::SessionDao::get_session_by_id(&self.db, session_id).await
    }

    pub async fn handle_watcher_event(&self, event: ClaudeWatcherEvent) -> Result<(), AppError> {
        match event {
            ClaudeWatcherEvent::SessionCreated(path) => {
                self.process_session_file(&path).await?;
            }
            ClaudeWatcherEvent::SessionModified(path) => {
                self.process_session_file(&path).await?;
            }
            ClaudeWatcherEvent::SessionDeleted(path) => {
                self.delete_session_record(&path).await?;
            }
        }
        Ok(())
    }

    async fn process_session_file(&self, file_path: &PathBuf) -> Result<(), AppError> {
        log::debug!("Processing session file: {:?}", file_path);

        // Parse the session file
        let parsed_data = match self.parser.parse_session_file(file_path) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("Failed to parse session file {:?}: {}", file_path, e);
                return Ok(()); // Continue processing other files
            }
        };

        // Store the parsed session data in the database
        self.store_session_data(&parsed_data).await?;

        log::debug!("Successfully processed session file: {:?}", file_path);
        Ok(())
    }

    async fn store_session_data(&self, parsed_data: &ParsedSessionData) -> Result<(), AppError> {
        // Convert the parsed session data to database records
        let session = &parsed_data.session;

        // Store session record
        dao::SessionDao::upsert_session(
            &self.db,
            &session.id,
            session.workspace_path.as_deref(),
            &session.start_time,
            session.prompt.as_deref(),
            session.commands.len() as i32,
            session.files.len() as i32,
        ).await?;

        // Store associated commands
        for command in &session.commands {
            dao::CommandDao::insert_command(&self.db, &session.id, command).await?;
        }

        // Store associated files
        for file in &session.files {
            dao::FileDao::insert_file(&self.db, &session.id, file).await?;
        }

        Ok(())
    }

    async fn delete_session_record(&self, file_path: &PathBuf) -> Result<(), AppError> {
        // Extract session ID from file path
        let session_id = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Delete from database
        dao::SessionDao::delete_session(&self.db, &session_id).await?;

        Ok(())
    }
}