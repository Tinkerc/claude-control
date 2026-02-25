#![allow(non_snake_case)]

use crate::services::analysis::{
    ProjectAnalyzer, WorkflowAnalyzer, ContentAnalyzer, SimilarityFinder,
};
use crate::session_manager;

/// Get project statistics for all or specific project
#[tauri::command]
pub async fn get_project_stats(
    project_dir: Option<String>,
) -> Result<Vec<crate::services::analysis::ProjectStats>, String> {
    let sessions = tauri::async_runtime::spawn_blocking(session_manager::scan_sessions)
        .await
        .map_err(|e| format!("Failed to scan sessions: {e}"))?;

    let filtered = if let Some(path) = project_dir {
        sessions.into_iter()
            .filter(|s| s.project_dir.as_ref().map(|p| p == &path).unwrap_or(false))
            .collect()
    } else {
        sessions
    };

    Ok(ProjectAnalyzer::analyze_by_project(&filtered))
}

/// Get workflow patterns for all or specific project
#[tauri::command]
pub async fn get_workflow_patterns(
    project_dir: Option<String>,
) -> Result<crate::services::analysis::WorkflowPatterns, String> {
    let sessions = tauri::async_runtime::spawn_blocking(session_manager::scan_sessions)
        .await
        .map_err(|e| format!("Failed to scan sessions: {e}"))?;

    let filtered = if let Some(path) = project_dir {
        sessions.into_iter()
            .filter(|s| s.project_dir.as_ref().map(|p| p == &path).unwrap_or(false))
            .collect()
    } else {
        sessions
    };

    Ok(WorkflowAnalyzer::detect_patterns(&filtered))
}

/// Get content analysis for all or specific project
#[tauri::command]
pub async fn get_content_analysis(
    project_dir: Option<String>,
) -> Result<crate::services::analysis::ContentAnalysis, String> {
    let sessions = tauri::async_runtime::spawn_blocking(session_manager::scan_sessions)
        .await
        .map_err(|e| format!("Failed to scan sessions: {e}"))?;

    let filtered = if let Some(path) = project_dir {
        sessions.into_iter()
            .filter(|s| s.project_dir.as_ref().map(|p| p == &path).unwrap_or(false))
            .collect()
    } else {
        sessions
    };

    Ok(ContentAnalyzer::analyze(&filtered))
}

/// Find sessions similar to the given session
#[tauri::command]
pub async fn find_similar_sessions(
    session_id: String,
    limit: Option<usize>,
) -> Result<Vec<crate::services::analysis::SimilarSession>, String> {
    let sessions = tauri::async_runtime::spawn_blocking(session_manager::scan_sessions)
        .await
        .map_err(|e| format!("Failed to scan sessions: {e}"))?;

    let target = sessions.iter()
        .find(|s| s.session_id == session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    Ok(SimilarityFinder::find_similar(target, &sessions, limit.unwrap_or(5)))
}

/// Get all insights combined
#[tauri::command]
pub async fn get_all_insights(
    project_dir: Option<String>,
) -> Result<crate::services::analysis::AllInsights, String> {
    let sessions = tauri::async_runtime::spawn_blocking(session_manager::scan_sessions)
        .await
        .map_err(|e| format!("Failed to scan sessions: {e}"))?;

    if sessions.is_empty() {
        return Err("No sessions available for analysis".to_string());
    }

    let filtered = if let Some(path) = project_dir {
        sessions.into_iter()
            .filter(|s| s.project_dir.as_ref().map(|p| p == &path).unwrap_or(false))
            .collect()
    } else {
        sessions
    };

    let total = filtered.len();

    Ok(crate::services::analysis::AllInsights {
        project_stats: ProjectAnalyzer::analyze_by_project(&filtered),
        workflow_patterns: WorkflowAnalyzer::detect_patterns(&filtered),
        content_analysis: ContentAnalyzer::analyze(&filtered),
        total_sessions_analyzed: total,
    })
}
