//! Agent Watcher Service
//!
//! This service integrates the ClaudeWatcher with the SessionService to
//! automatically monitor and parse Claude Code session files.

use crate::config::get_claude_config_dir;
use crate::services::SessionService;
use crate::watcher::ClaudeWatcher;
use crate::error::AppError;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Agent Watcher Service
///
/// This service monitors Claude Code session files and automatically
/// processes them when they are created or modified.
pub struct AgentWatcherService {
    _handle: tokio::task::JoinHandle<()>,
}

// AgentWatcherService is Send + Sync because it only contains a JoinHandle
unsafe impl Send for AgentWatcherService {}
unsafe impl Sync for AgentWatcherService {}

impl AgentWatcherService {
    /// Create and start a new agent watcher service
    ///
    /// This will:
    /// 1. Create a ClaudeWatcher pointing to ~/.claude/sessions/
    /// 2. Process all existing session files
    /// 3. Start watching for new/modified session files
    pub fn start(db: Arc<crate::database::Database>) -> Result<Self, AppError> {
        let sessions_dir = get_claude_config_dir().join("sessions");

        // Ensure the sessions directory exists
        if !sessions_dir.exists() {
            log::info!("Claude sessions directory does not exist: {:?}", sessions_dir);
            // Create the directory if it doesn't exist
            std::fs::create_dir_all(&sessions_dir)
                .map_err(|e| AppError::io(&sessions_dir, e))?;
        }

        log::info!("Starting Claude Agent Watcher for directory: {:?}", sessions_dir);

        // Create channel for watcher events
        let (event_sender, mut event_receiver) = mpsc::channel::<crate::watcher::ClaudeWatcherEvent>(100);

        // Create the watcher with 500ms debounce
        let mut watcher = ClaudeWatcher::new(Duration::from_millis(500), event_sender)
            .map_err(|e| AppError::Database(format!("Failed to create watcher: {}", e)))?;

        // Start watching the sessions directory
        watcher.watch_sessions_directory(&sessions_dir)
            .map_err(|e| AppError::Database(format!("Failed to watch sessions directory: {}", e)))?;

        let session_service = SessionService::new(db.clone());

        // Spawn a background task to process watcher events
        let handle = tokio::spawn(async move {
            log::info!("Agent watcher event loop started");

            // Process existing session files first
            if let Err(e) = Self::process_existing_sessions(&session_service, &sessions_dir).await {
                log::error!("Failed to process existing sessions: {}", e);
            }

            // Then process new events
            while let Some(event) = event_receiver.recv().await {
                log::debug!("Received watcher event: {:?}", event);

                // Handle the event in the session service
                if let Err(e) = session_service.handle_watcher_event(event).await {
                    log::error!("Failed to handle watcher event: {}", e);
                }
            }

            log::info!("Agent watcher event loop stopped");
        });

        Ok(Self {
            _handle: handle,
        })
    }

    /// Process all existing session files in the sessions directory
    async fn process_existing_sessions(
        session_service: &SessionService,
        sessions_dir: &PathBuf,
    ) -> Result<(), AppError> {
        log::info!("Processing existing session files in: {:?}", sessions_dir);

        let entries = std::fs::read_dir(sessions_dir)
            .map_err(|e| AppError::io(sessions_dir, e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| AppError::io(sessions_dir, e))?;
            let path = entry.path();

            // Only process JSON files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                log::debug!("Processing existing session file: {:?}", path);

                // Process this file as a session created event
                let event = crate::watcher::ClaudeWatcherEvent::SessionCreated(path);
                if let Err(e) = session_service.handle_watcher_event(event).await {
                    log::warn!("Failed to process session file {:?}: {}", path, e);
                } else {
                    count += 1;
                }
            }
        }

        log::info!("Processed {} existing session files", count);
        Ok(())
    }
}

/// Manually trigger a re-scan of all session files
///
/// This can be called by the user to force a refresh of all session data.
pub async fn rescan_all_sessions(
    db: Arc<crate::database::Database>,
) -> Result<(), AppError> {
    let sessions_dir = get_claude_config_dir().join("sessions");
    let session_service = SessionService::new(db);

    log::info!("Re-scanning all session files in: {:?}", sessions_dir);
    AgentWatcherService::process_existing_sessions(&session_service, &sessions_dir).await
}
