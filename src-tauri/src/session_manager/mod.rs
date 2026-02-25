pub mod providers;
pub mod terminal;

use serde::{Deserialize, Serialize};
use std::path::Path;

use providers::{claude, codex, gemini, openclaw, opencode};

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionStats {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_minutes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_message_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant_message_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools_used: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files_modified: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands_executed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMeta {
    pub provider_id: String,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<SessionStats>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<i64>,
}

pub fn scan_sessions() -> Vec<SessionMeta> {
    let mut sessions = Vec::new();
    sessions.extend(codex::scan_sessions());
    sessions.extend(claude::scan_sessions());
    sessions.extend(opencode::scan_sessions());
    sessions.extend(openclaw::scan_sessions());
    sessions.extend(gemini::scan_sessions());

    sessions.sort_by(|a, b| {
        let a_ts = a.last_active_at.or(a.created_at).unwrap_or(0);
        let b_ts = b.last_active_at.or(b.created_at).unwrap_or(0);
        b_ts.cmp(&a_ts)
    });

    sessions
}

pub fn load_messages(provider_id: &str, source_path: &str) -> Result<Vec<SessionMessage>, String> {
    let path = Path::new(source_path);
    match provider_id {
        "codex" => codex::load_messages(path),
        "claude" => claude::load_messages(path),
        "opencode" => opencode::load_messages(path),
        "openclaw" => openclaw::load_messages(path),
        "gemini" => gemini::load_messages(path),
        _ => Err(format!("Unsupported provider: {provider_id}")),
    }
}

// ============================================================================
// Phase 2: Enhanced Search
// ============================================================================

/// Search query options for filtering sessions
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionSearchQuery {
    /// Search keyword (matches title, summary, session ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    /// Search in files modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<String>,
    /// Search in commands executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<String>,
    /// Filter by tools used (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<String>,
    /// Search in project directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Filter by provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Filter by time range (timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    /// Filter by time range (timestamp in milliseconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

/// Search results with matched sessions and metadata
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSearchResult {
    pub sessions: Vec<SessionMeta>,
    pub total: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_commands: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_tools: Option<Vec<String>>,
}

/// Search sessions by files, commands, tools, and time range
pub fn search_sessions(query: &SessionSearchQuery) -> SessionSearchResult {
    let all_sessions = scan_sessions();
    
    let mut matched_sessions = Vec::new();
    let mut matched_files = std::collections::HashSet::new();
    let mut matched_commands = std::collections::HashSet::new();
    let mut matched_tools = std::collections::HashSet::new();

    for session in all_sessions {
        if matches_query(&session, query) {
            // Collect matched metadata
            if let Some(files) = session.stats.as_ref().and_then(|s| s.files_modified.as_ref()) {
                for file in files {
                    matched_files.insert(file.clone());
                }
            }
            if let Some(commands) = session.stats.as_ref().and_then(|s| s.commands_executed.as_ref()) {
                for command in commands {
                    matched_commands.insert(command.clone());
                }
            }
            if let Some(tools) = session.stats.as_ref().and_then(|s| s.tools_used.as_ref()) {
                for tool in tools {
                    matched_tools.insert(tool.clone());
                }
            }
            matched_sessions.push(session);
        }
    }

    // Sort by last_active_at descending
    matched_sessions.sort_by(|a, b| {
        let a_ts = a.last_active_at.or(a.created_at).unwrap_or(0);
        let b_ts = b.last_active_at.or(b.created_at).unwrap_or(0);
        b_ts.cmp(&a_ts)
    });

    let total_count = matched_sessions.len();

    SessionSearchResult {
        sessions: matched_sessions,
        total: total_count,
        matched_files: if !matched_files.is_empty() {
            Some(matched_files.into_iter().collect())
        } else {
            None
        },
        matched_commands: if !matched_commands.is_empty() {
            Some(matched_commands.into_iter().collect())
        } else {
            None
        },
        matched_tools: if !matched_tools.is_empty() {
            Some(matched_tools.into_iter().collect())
        } else {
            None
        },
    }
}

/// Check if a session matches the search query
fn matches_query(session: &SessionMeta, query: &SessionSearchQuery) -> bool {
    // Filter by keyword (matches title, summary, session ID)
    if let Some(ref keyword) = query.keyword {
        let keyword_lower = keyword.to_lowercase();
        let matches_keyword = session
            .title
            .as_ref()
            .map(|t| t.to_lowercase().contains(&keyword_lower))
            .unwrap_or(false)
            || session
                .summary
                .as_ref()
                .map(|s| s.to_lowercase().contains(&keyword_lower))
                .unwrap_or(false)
            || session
                .session_id
                .to_lowercase()
                .contains(&keyword_lower);
        
        if !matches_keyword {
            return false;
        }
    }

    // Filter by project directory
    if let Some(ref project_query) = query.project {
        let project_dir = session.project_dir.as_ref();
        if let Some(dir) = project_dir {
            let project_lower = project_query.to_lowercase();
            if !dir.to_lowercase().contains(&project_lower) {
                return false;
            }
        } else {
            return false;
        }
    }

    // Filter by provider
    if let Some(ref provider) = query.provider {
        if session.provider_id != *provider {
            return false;
        }
    }

    // Filter by time range
    let session_time = session.last_active_at.or(session.created_at).unwrap_or(0);
    if let Some(start) = query.start_time {
        if session_time < start {
            return false;
        }
    }
    if let Some(end) = query.end_time {
        if session_time > end {
            return false;
        }
    }

    // Filter by files
    if let Some(ref files_query) = query.files {
        let files = session
            .stats
            .as_ref()
            .and_then(|s| s.files_modified.as_ref());

        if let Some(files) = files {
            let files_lower = files_query.to_lowercase();
            let has_match = files.iter().any(|f| f.to_lowercase().contains(&files_lower));
            if !has_match {
                return false;
            }
        } else {
            return false;
        }
    }

    // Filter by commands
    if let Some(ref commands_query) = query.commands {
        let commands = session
            .stats
            .as_ref()
            .and_then(|s| s.commands_executed.as_ref());

        if let Some(commands) = commands {
            let commands_lower = commands_query.to_lowercase();
            let has_match = commands.iter().any(|c| c.to_lowercase().contains(&commands_lower));
            if !has_match {
                return false;
            }
        } else {
            return false;
        }
    }

    // Filter by tools
    if let Some(ref tools_query) = query.tools {
        let tools = session
            .stats
            .as_ref()
            .and_then(|s| s.tools_used.as_ref());

        if let Some(tools) = tools {
            // Support comma-separated tool names
            let tool_filters: Vec<&str> = tools_query.split(',').map(|s| s.trim()).collect();
            let has_match = tool_filters.iter().any(|filter| {
                let filter_lower = filter.to_lowercase();
                tools.iter().any(|t| t.to_lowercase().contains(&filter_lower))
            });
            if !has_match {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}
