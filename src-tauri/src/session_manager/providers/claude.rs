use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::config::get_claude_config_dir;
use crate::session_manager::{SessionMessage, SessionMeta, SessionStats, TokenUsage};

use super::utils::{extract_text, parse_timestamp_to_ms, path_basename, truncate_summary};

const PROVIDER_ID: &str = "claude";

pub fn scan_sessions() -> Vec<SessionMeta> {
    let root = get_claude_config_dir().join("projects");
    let mut files = Vec::new();
    collect_jsonl_files(&root, &mut files);

    let mut sessions = Vec::new();
    for path in files {
        if let Some(meta) = parse_session(&path) {
            sessions.push(meta);
        }
    }

    sessions
}

pub fn load_messages(path: &Path) -> Result<Vec<SessionMessage>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open session file: {e}"))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(value) => value,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&line) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        if value.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }

        let message = match value.get("message") {
            Some(message) => message,
            None => continue,
        };

        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let content = message.get("content").map(extract_text).unwrap_or_default();
        if content.trim().is_empty() {
            continue;
        }

        let ts = value.get("timestamp").and_then(parse_timestamp_to_ms);

        messages.push(SessionMessage { role, content, ts });
    }

    Ok(messages)
}

fn parse_session(path: &Path) -> Option<SessionMeta> {
    if is_agent_session(path) {
        return None;
    }

    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut session_id: Option<String> = None;
    let mut project_dir: Option<String> = None;
    let mut created_at: Option<i64> = None;
    let mut last_active_at: Option<i64> = None;
    let mut summary: Option<String> = None;
    
    // Stats tracking
    let mut user_message_count = 0usize;
    let mut assistant_message_count = 0usize;
    let mut tools_used: HashSet<String> = HashSet::new();
    let mut files_modified: HashSet<String> = HashSet::new();
    let mut commands_executed: HashSet<String> = HashSet::new();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut model: Option<String> = None;
    let mut topic: Option<String> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(value) => value,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&line) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        if session_id.is_none() {
            session_id = value
                .get("sessionId")
                .and_then(Value::as_str)
                .map(|s| s.to_string());
        }

        if project_dir.is_none() {
            project_dir = value
                .get("cwd")
                .and_then(Value::as_str)
                .map(|s| s.to_string());
        }

        if let Some(ts) = value.get("timestamp").and_then(parse_timestamp_to_ms) {
            if created_at.is_none() {
                created_at = Some(ts);
            }
            last_active_at = Some(ts);
        }

        // Extract model from metadata
        if model.is_none() {
            if let Some(msg) = value.get("message") {
                if let Some(model_val) = msg.get("model") {
                    if let Some(model_str) = model_val.as_str() {
                        model = Some(model_str.to_string());
                    }
                }
            }
        }

        if value.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }

        let message = match value.get("message") {
            Some(message) => message,
            None => continue,
        };

        let text = message.get("content").map(extract_text).unwrap_or_default();
        if text.trim().is_empty() {
            continue;
        }
        
        // Track first user message as topic
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        
        if topic.is_none() && role == "user" {
            topic = Some(truncate_summary(&text, 100));
        }
        
        summary = Some(text.clone());

        // Count messages by role
        if role == "user" {
            user_message_count += 1;
        } else if role == "assistant" {
            assistant_message_count += 1;
        }

        // Extract tool usage from assistant messages
        if role == "assistant" {
            if let Some(content) = message.get("content") {
                if let Some(blocks) = content.as_array() {
                    for block in blocks {
                        if let Some(tool_type) = block.get("type").and_then(Value::as_str) {
                            // Track tool use
                            if tool_type == "tool_use" {
                                if let Some(tool_name) = block.get("name").and_then(Value::as_str) {
                                    tools_used.insert(tool_name.to_string());
                                }
                            }
                            // Track tool result (for files modified inference)
                            if tool_type == "tool_result" {
                                let _tool_use_id = block.get("tool_use_id").and_then(Value::as_str);
                                // We can track which tools got results
                            }
                        }
                    }
                }
            }
        }
        
        // Extract tool use from user messages (tool_result blocks)
        if role == "user" {
            if let Some(content) = message.get("content") {
                if let Some(blocks) = content.as_array() {
                    for block in blocks {
                        if let Some(tool_type) = block.get("type").and_then(Value::as_str) {
                            if tool_type == "tool_result" {
                                // Extract command from bash tool results
                                if let Some(input) = block.get("input") {
                                    if let Some(cmd) = input.get("command").and_then(Value::as_str) {
                                        commands_executed.insert(cmd.to_string());
                                    }
                                }
                                // Extract file paths from read/write tool results
                                if let Some(input) = block.get("input") {
                                    if let Some(file_path) = input.get("file_path").and_then(Value::as_str) {
                                        files_modified.insert(file_path.to_string());
                                    }
                                    if let Some(path) = input.get("path").and_then(Value::as_str) {
                                        files_modified.insert(path.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Extract token usage from assistant messages
        if role == "assistant" {
            if let Some(usage) = message.get("usage") {
                if let Some(input) = usage.get("input_tokens").and_then(Value::as_u64) {
                    input_tokens = input_tokens.saturating_add(input);
                }
                if let Some(output) = usage.get("output_tokens").and_then(Value::as_u64) {
                    output_tokens = output_tokens.saturating_add(output);
                }
            }
        }
    }

    let session_id = session_id.or_else(|| infer_session_id_from_filename(path));
    let session_id = session_id?;

    let title = project_dir
        .as_deref()
        .and_then(path_basename)
        .map(|value| value.to_string());

    let summary = summary.map(|text| truncate_summary(&text, 160));
    
    // Calculate duration
    let duration_minutes = match (created_at, last_active_at) {
        (Some(start), Some(end)) => Some(((end - start) / 60000) as u64),
        _ => None,
    };

    let stats = SessionStats {
        duration_minutes,
        message_count: Some(user_message_count + assistant_message_count),
        user_message_count: Some(user_message_count),
        assistant_message_count: Some(assistant_message_count),
        tools_used: if tools_used.is_empty() {
            None
        } else {
            Some(tools_used.into_iter().collect())
        },
        files_modified: if files_modified.is_empty() {
            None
        } else {
            // Limit to first 10 files to keep data manageable
            Some(files_modified.into_iter().take(10).collect())
        },
        commands_executed: if commands_executed.is_empty() {
            None
        } else {
            // Limit to first 10 commands
            Some(commands_executed.into_iter().take(10).collect())
        },
        token_usage: if input_tokens == 0 && output_tokens == 0 {
            None
        } else {
            Some(TokenUsage {
                input_tokens: Some(input_tokens),
                output_tokens: Some(output_tokens),
            })
        },
        model,
        topic,
    };

    Some(SessionMeta {
        provider_id: PROVIDER_ID.to_string(),
        session_id: session_id.clone(),
        title,
        summary,
        project_dir,
        created_at,
        last_active_at,
        source_path: Some(path.to_string_lossy().to_string()),
        resume_command: Some(format!("claude --resume {session_id}")),
        stats: Some(stats),
    })
}

fn is_agent_session(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("agent-"))
        .unwrap_or(false)
}

fn infer_session_id_from_filename(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem.to_string())
}

fn collect_jsonl_files(root: &Path, files: &mut Vec<PathBuf>) {
    if !root.exists() {
        return;
    }

    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }
}
