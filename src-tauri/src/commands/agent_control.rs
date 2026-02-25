use crate::services::{SessionService, MemoryService, InsightService};
use crate::store::AppState;
use crate::error::AppError;

/// Get all sessions from the database
#[tauri::command]
pub async fn get_all_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<crate::database::dao::sessions::Session>, String> {
    let service = SessionService::new(state.db.clone());
    service.get_all_sessions().await
        .map_err(|e| e.to_string())
}

/// Get sessions for a specific workspace
#[tauri::command]
pub async fn get_workspace_sessions(
    state: tauri::State<'_, AppState>,
    workspace: String,
) -> Result<Vec<crate::database::dao::sessions::Session>, String> {
    let service = SessionService::new(state.db.clone());
    service.get_sessions_by_workspace(&workspace).await
        .map_err(|e| e.to_string())
}

/// Get a specific session by ID
#[tauri::command]
pub async fn get_session_by_id(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Option<crate::database::dao::sessions::Session>, String> {
    let service = SessionService::new(state.db.clone());
    service.get_session_by_id(&session_id).await
        .map_err(|e| e.to_string())
}

/// Get session timeline for the specified number of days
#[tauri::command]
pub async fn get_session_timeline(
    state: tauri::State<'_, AppState>,
    days: i32,
) -> Result<Vec<crate::database::dao::sessions::Session>, String> {
    let sessions = crate::database::dao::sessions::SessionDao::get_session_timeline(&state.db, days).await
        .map_err(|e| e.to_string())?;
    Ok(sessions)
}

/// Get memory entries for a workspace or all memories
#[tauri::command]
pub async fn get_memory_entries(
    state: tauri::State<'_, AppState>,
    workspace: Option<String>,
) -> Result<Vec<crate::services::memory_service::MemoryEntry>, String> {
    let service = MemoryService::new(state.db.clone());
    service.get_memory_entries(workspace.as_deref()).await
        .map_err(|e| e.to_string())
}

/// Get a specific memory entry by key
#[tauri::command]
pub async fn get_memory_entry(
    state: tauri::State<'_, AppState>,
    key: String,
    workspace: Option<String>,
) -> Result<Option<crate::services::memory_service::MemoryEntry>, String> {
    let service = MemoryService::new(state.db.clone());
    service.get_memory_entry(&key, workspace.as_deref()).await
        .map_err(|e| e.to_string())
}

/// Update or create a memory entry
#[tauri::command]
pub async fn update_memory_entry(
    state: tauri::State<'_, AppState>,
    key: String,
    value: String,
    workspace: Option<String>,
    confidence: f64,
) -> Result<(), String> {
    let service = MemoryService::new(state.db.clone());
    service.update_memory_entry(&key, &value, workspace, confidence).await
        .map_err(|e| e.to_string())
}

/// Delete a memory entry
#[tauri::command]
pub async fn delete_memory_entry(
    state: tauri::State<'_, AppState>,
    key: String,
    workspace: Option<String>,
) -> Result<(), String> {
    let service = MemoryService::new(state.db.clone());
    service.delete_memory_entry(&key, workspace.as_deref()).await
        .map_err(|e| e.to_string())
}

/// Get top memories by confidence
#[tauri::command]
pub async fn get_top_memories(
    state: tauri::State<'_, AppState>,
    workspace: Option<String>,
    limit: usize,
) -> Result<Vec<crate::services::memory_service::MemoryEntry>, String> {
    let service = MemoryService::new(state.db.clone());
    service.get_top_memories(workspace.as_deref(), limit).await
        .map_err(|e| e.to_string())
}

/// Extract memory from a specific session
#[tauri::command]
pub async fn extract_session_memory(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let service = MemoryService::new(state.db.clone());
    service.extract_memory_from_session(&session_id).await
        .map_err(|e| e.to_string())
}

/// Get session statistics
#[tauri::command]
pub async fn get_session_stats(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::services::insight_service::SessionStat>, String> {
    let service = InsightService::new(state.db.clone());
    service.get_session_stats().await
        .map_err(|e| e.to_string())
}

/// Get command usage statistics
#[tauri::command]
pub async fn get_command_statistics(
    state: tauri::State<'_, AppState>,
    workspace: Option<String>,
) -> Result<Vec<crate::services::insight_service::CommandStat>, String> {
    let service = InsightService::new(state.db.clone());
    service.get_command_statistics(workspace.as_deref()).await
        .map_err(|e| e.to_string())
}

/// Get file access statistics
#[tauri::command]
pub async fn get_file_statistics(
    state: tauri::State<'_, AppState>,
    workspace: Option<String>,
) -> Result<Vec<crate::services::insight_service::FileStat>, String> {
    let service = InsightService::new(state.db.clone());
    service.get_file_statistics(workspace.as_deref()).await
        .map_err(|e| e.to_string())
}

/// Get prompt usage statistics
#[tauri::command]
pub async fn get_prompt_statistics(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::services::insight_service::PromptStat>, String> {
    let service = InsightService::new(state.db.clone());
    service.get_prompt_statistics().await
        .map_err(|e| e.to_string())
}

/// Get workspace activity statistics
#[tauri::command]
pub async fn get_workspace_activity(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(String, i32)>, String> {
    let service = InsightService::new(state.db.clone());
    service.get_workspace_activity().await
        .map_err(|e| e.to_string())
}

/// Get active sessions count
#[tauri::command]
pub async fn get_active_sessions_count(
    state: tauri::State<'_, AppState>,
) -> Result<i32, String> {
    let service = InsightService::new(state.db.clone());
    service.get_active_sessions_count().await
        .map_err(|e| e.to_string())
}

/// Get commands for a specific session
#[tauri::command]
pub async fn get_session_commands(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<crate::database::dao::commands::Command>, String> {
    crate::database::dao::commands::CommandDao::get_commands_by_session(&state.db, &session_id).await
        .map_err(|e| e.to_string())
}

/// Get files for a specific session
#[tauri::command]
pub async fn get_session_files(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<crate::database::dao::files::FileRecord>, String> {
    crate::database::dao::files::FileDao::get_files_by_session(&state.db, &session_id).await
        .map_err(|e| e.to_string())
}

/// Manually trigger a re-scan of all session files
#[tauri::command]
pub async fn rescan_all_sessions(state: tauri::State<'_, AppState>) -> Result<(), String> {
    crate::services::rescan_all_sessions(state.db.clone()).await
        .map_err(|e| e.to_string())
}
