use chrono::{DateTime, Utc};
use regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub workspace_path: Option<String>,
    pub start_time: DateTime<Utc>,
    pub prompt: Option<String>,
    pub commands: Vec<String>,
    pub files: Vec<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSessionData {
    pub session: Session,
    pub raw_content: String,
}

pub struct SessionParser;

impl SessionParser {
    pub fn new() -> Self {
        SessionParser
    }

    pub fn parse_session_file(&self, file_path: &PathBuf) -> Result<ParsedSessionData, Box<dyn std::error::Error>> {
        // Read the session file
        let content = std::fs::read_to_string(file_path)?;

        // Attempt to parse as JSON
        let parsed_value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(val) => val,
            Err(e) => {
                log::error!("Failed to parse session file as JSON: {}, Error: {}", file_path.display(), e);
                return Err(Box::new(e));
            }
        };

        // Extract key information from the parsed JSON
        let session = self.extract_session_data(&parsed_value, file_path)?;

        Ok(ParsedSessionData {
            session,
            raw_content: content,
        })
    }

    fn extract_session_data(&self, json_value: &serde_json::Value, file_path: &PathBuf) -> Result<Session, Box<dyn std::error::Error>> {
        // Extract the session ID (use filename as ID if available)
        let id = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown_session")
            .to_string();

        // Extract workspace path if available
        let workspace_path = self.extract_workspace_path(json_value);

        // Extract start time - try multiple possible fields
        let start_time = self.extract_timestamp(json_value)?;

        // Extract prompt if available
        let prompt = self.extract_prompt(json_value);

        // Extract commands from the session
        let commands = self.extract_commands(json_value);

        // Extract files accessed in the session
        let files = self.extract_files(json_value);

        // Extract summary if available
        let summary = self.extract_summary(json_value);

        Ok(Session {
            id,
            workspace_path,
            start_time,
            prompt,
            commands,
            files,
            summary,
        })
    }

    fn extract_workspace_path(&self, json_value: &serde_json::Value) -> Option<String> {
        // Look for workspace-related fields in various possible locations
        [
            "workspace",
            "workspace_path",
            "current_workspace",
            "project_path",
            "working_directory",
        ]
        .iter()
        .find_map(|&field| {
            json_value
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
    }

    fn extract_timestamp(&self, json_value: &serde_json::Value) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
        // Try different timestamp fields that might exist in the session
        let timestamp_str = [
            "created_at",
            "timestamp",
            "start_time",
            "createdAt",
            "startTime",
            "date",
        ]
        .iter()
        .find_map(|&field| {
            json_value
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

        if let Some(ts_str) = timestamp_str {
            // Try to parse as ISO 8601 timestamp
            match DateTime::parse_from_rfc3339(&ts_str) {
                Ok(dt) => return Ok(dt.with_timezone(&Utc)),
                Err(_) => {
                    // If RFC3339 parsing fails, try other formats
                    // Common alternative: "YYYY-MM-DD HH:MM:SS"
                    if let Ok(dt) = DateTime::parse_from_str(&ts_str, "%Y-%m-%d %H:%M:%S%.f%z") {
                        return Ok(dt.with_timezone(&Utc));
                    }
                    // Another common format: Unix timestamp
                    if let Ok(timestamp) = ts_str.parse::<i64>() {
                        return Ok(DateTime::<Utc>::from_timestamp(timestamp, 0).ok_or_else(|| {
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))
                        })?);
                    }
                }
            }
        }

        // If no explicit timestamp found, use file modification time as fallback
        use std::fs;
        let metadata = fs::metadata(
            json_value
                .as_object()
                .and_then(|obj| obj.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or(""), // This won't work - we need a different approach
        ).unwrap_or_else(|_| {
            // Use current time as fallback if we can't get file metadata
            std::fs::metadata(file_path).unwrap()
        });

        let modified = metadata.modified()
            .unwrap_or_else(|_| std::time::SystemTime::now());

        Ok(DateTime::<Utc>::from(modified))
    }

    fn extract_prompt(&self, json_value: &serde_json::Value) -> Option<String> {
        // Look for prompt or message content in various possible locations
        [
            "prompt",
            "initial_prompt",
            "user_input",
            "input",
            "message",
            "query",
        ]
        .iter()
        .find_map(|&field| {
            json_value
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            // Look for prompts in messages array
            if let Some(messages) = json_value.get("messages").and_then(|v| v.as_array()) {
                for message in messages {
                    if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                        return Some(content.to_string());
                    }
                    if let Some(text) = message.get("text").and_then(|t| t.as_str()) {
                        return Some(text.to_string());
                    }
                }
            }
            None
        })
    }

    fn extract_commands(&self, json_value: &serde_json::Value) -> Vec<String> {
        let mut commands = Vec::new();

        // Look for commands in various possible locations
        if let Some(cmd_val) = json_value.get("commands").or_else(|| json_value.get("command")) {
            if let Some(cmd_array) = cmd_val.as_array() {
                for cmd in cmd_array {
                    if let Some(cmd_str) = cmd.as_str() {
                        commands.push(cmd_str.to_string());
                    }
                }
            } else if let Some(cmd_str) = cmd_val.as_str() {
                commands.push(cmd_str.to_string());
            }
        }

        // Also look for shell commands in messages
        if let Some(messages) = json_value.get("messages").and_then(|v| v.as_array()) {
            for message in messages {
                // Look for tool calls or command execution
                if let Some(tool_calls) = message.get("tool_calls").and_then(|v| v.as_array()) {
                    for call in tool_calls {
                        if let Some(function) = call.get("function") {
                            if let Some(name) = function.get("name").and_then(|n| n.as_str()) {
                                if name == "execute_command" || name.contains("shell") || name.contains("command") {
                                    if let Some(args_str) = function.get("arguments").and_then(|a| a.as_str()) {
                                        if let Ok(args) = serde_json::from_str::<serde_json::Value>(args_str) {
                                            if let Some(command) = args.get("command").and_then(|c| c.as_str()) {
                                                commands.push(command.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Look for inline commands in message content
                if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                    // Look for common shell command patterns
                    let shell_patterns = [
                        r"```\w*sh(?:ell)?\n([^`]+)",
                        r"`([^`]+)`",
                        r"\$(\w+)\s+(.+?)(?:\s|$)",
                    ];

                    for pattern in &shell_patterns {
                        if let Ok(regex) = regex::Regex::new(pattern) {
                            for cap in regex.captures_iter(content) {
                                if let Some(cmd) = cap.get(1) {
                                    let cmd_str = cmd.as_str().trim();
                                    if !cmd_str.is_empty() && !commands.contains(&cmd_str.to_string()) {
                                        commands.push(cmd_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        commands
    }

    fn extract_files(&self, json_value: &serde_json::Value) -> Vec<String> {
        let mut files = Vec::new();

        // Look for files in various possible locations
        if let Some(files_val) = json_value.get("files").or_else(|| json_value.get("file")) {
            if let Some(files_array) = files_val.as_array() {
                for file in files_array {
                    if let Some(file_str) = file.as_str() {
                        files.push(file_str.to_string());
                    }
                }
            } else if let Some(file_str) = files_val.as_str() {
                files.push(file_str.to_string());
            }
        }

        // Also look for files mentioned in messages
        if let Some(messages) = json_value.get("messages").and_then(|v| v.as_array()) {
            for message in messages {
                if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                    // Look for common file patterns
                    if let Ok(file_regex) = regex::Regex::new(r#"["']?([^\s"'<>|&;{}]+?\.(?:txt|rs|js|ts|jsx|tsx|py|java|cpp|c|h|cs|go|rb|php|html|css|json|yaml|yml|toml|xml|md|sql|sh|bat|ps1))["']?"#) {
                        for cap in file_regex.captures_iter(content) {
                            if let Some(file_match) = cap.get(1) {
                                let file_str = file_match.as_str();
                                if !files.contains(&file_str.to_string()) {
                                    files.push(file_str.to_string());
                                }
                            }
                        }
                    }

                    // Look for file paths in general
                    if let Ok(path_regex) = regex::Regex::new(r#"["']?((?:/[^\s"'<>|&;{}]+)+\.[a-zA-Z0-9]+)["']?"#) {
                        for cap in path_regex.captures_iter(content) {
                            if let Some(path_match) = cap.get(1) {
                                let path_str = path_match.as_str();
                                if !files.contains(&path_str.to_string()) {
                                    files.push(path_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        files
    }

    fn extract_summary(&self, json_value: &serde_json::Value) -> Option<String> {
        // Look for summary or conclusion fields
        [
            "summary",
            "conclusion",
            "result",
            "outcome",
            "summary_text",
            "final_message",
        ]
        .iter()
        .find_map(|&field| {
            json_value
                .get(field)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
    }
}