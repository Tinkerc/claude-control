use std::collections::{HashMap, HashSet};
use crate::session_manager::SessionMeta;

/// Project-level statistics aggregator
pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    /// Analyze sessions grouped by project path
    pub fn analyze_by_project(sessions: &[SessionMeta]) -> Vec<ProjectStats> {
        let mut project_map: HashMap<String, Vec<&SessionMeta>> = HashMap::new();

        // Group by project
        for session in sessions {
            let project_path = session.project_dir.clone().unwrap_or_else(|| "unknown".to_string());
            project_map
                .entry(project_path)
                .or_default()
                .push(session);
        }

        // Analyze each project
        project_map
            .into_iter()
            .map(|(path, sessions)| Self::analyze_single_project(&path, &sessions))
            .collect()
    }

    fn analyze_single_project(path: &str, sessions: &[&SessionMeta]) -> ProjectStats {
        let mut cmd_counts: HashMap<String, usize> = HashMap::new();
        let mut file_counts: HashMap<String, usize> = HashMap::new();
        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        let mut total_duration = 0u64;

        for session in sessions {
            if let Some(stats) = &session.stats {
                total_duration += stats.duration_minutes.unwrap_or(0);

                if let Some(cmds) = &stats.commands_executed {
                    for cmd in cmds {
                        *cmd_counts.entry(cmd.clone()).or_insert(0) += 1;
                    }
                }
                if let Some(files) = &stats.files_modified {
                    for file in files {
                        *file_counts.entry(file.clone()).or_insert(0) += 1;
                    }
                }
                if let Some(tools) = &stats.tools_used {
                    for tool in tools {
                        *tool_counts.entry(tool.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let total_cmds: usize = cmd_counts.values().sum();
        let total_files: usize = file_counts.values().sum();
        let total_tools: usize = tool_counts.values().sum();

        ProjectStats {
            project_dir: path.to_string(),
            session_count: sessions.len(),
            total_duration_minutes: total_duration as i64,
            top_commands: Self::top_n_commands(cmd_counts, total_cmds, 10),
            top_files: Self::top_n_files(file_counts, total_files, 10),
            top_tools: Self::top_n_tools(tool_counts, total_tools, 10),
            avg_session_duration: if sessions.is_empty() {
                0.0
            } else {
                total_duration as f64 / sessions.len() as f64
            },
        }
    }

    fn top_n_commands(counts: HashMap<String, usize>, total: usize, n: usize) -> Vec<CommandFrequency> {
        let mut items: Vec<_> = counts
            .into_iter()
            .map(|(command, count)| CommandFrequency {
                command,
                count,
                percentage: if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        items.sort_by(|a, b| b.count.cmp(&a.count));
        items.truncate(n);
        items
    }

    fn top_n_files(counts: HashMap<String, usize>, total: usize, n: usize) -> Vec<FileFrequency> {
        let mut items: Vec<_> = counts
            .into_iter()
            .map(|(file, count)| FileFrequency {
                file,
                edit_count: count,
                percentage: if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        items.sort_by(|a, b| b.edit_count.cmp(&a.edit_count));
        items.truncate(n);
        items
    }

    fn top_n_tools(counts: HashMap<String, usize>, total: usize, n: usize) -> Vec<ToolFrequency> {
        let mut items: Vec<_> = counts
            .into_iter()
            .map(|(tool, count)| ToolFrequency {
                tool,
                usage_count: count,
                percentage: if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        items.sort_by(|a, b| b.usage_count.cmp(&a.usage_count));
        items.truncate(n);
        items
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectStats {
    pub project_dir: String,
    pub session_count: usize,
    pub total_duration_minutes: i64,
    pub top_commands: Vec<CommandFrequency>,
    pub top_files: Vec<FileFrequency>,
    pub top_tools: Vec<ToolFrequency>,
    pub avg_session_duration: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CommandFrequency {
    pub command: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct FileFrequency {
    pub file: String,
    pub edit_count: usize,
    pub percentage: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ToolFrequency {
    pub tool: String,
    pub usage_count: usize,
    pub percentage: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_manager::{SessionMeta, SessionStats};

    fn create_test_session(
        project: &str,
        commands: Vec<&str>,
        files: Vec<&str>,
        duration: u64,
    ) -> SessionMeta {
        SessionMeta {
            provider_id: "claude".to_string(),
            session_id: "test-id".to_string(),
            project_dir: Some(project.to_string()),
            stats: Some(SessionStats {
                commands_executed: Some(commands.into_iter().map(String::from).collect()),
                files_modified: Some(files.into_iter().map(String::from).collect()),
                duration_minutes: Some(duration),
                tools_used: Some(vec!["Write".to_string(), "Read".to_string()]),
                message_count: Some(10),
                user_message_count: Some(5),
                assistant_message_count: Some(5),
                token_usage: None,
                model: None,
                topic: None,
            }),
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            last_active_at: Some(chrono::Utc::now().timestamp_millis()),
            title: None,
            summary: None,
            source_path: None,
            resume_command: None,
        }
    }

    #[test]
    fn test_group_by_project() {
        let sessions = vec![
            create_test_session("/proj/a", vec!["cargo test"], vec!["src/a.rs"], 30),
            create_test_session("/proj/a", vec!["cargo build"], vec!["src/b.rs"], 20),
            create_test_session("/proj/b", vec!["npm test"], vec!["src/a.ts"], 15),
        ];

        let result = ProjectAnalyzer::analyze_by_project(&sessions);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].project_dir, "/proj/a");
        assert_eq!(result[0].session_count, 2);
        assert_eq!(result[1].project_dir, "/proj/b");
        assert_eq!(result[1].session_count, 1);
    }

    #[test]
    fn test_top_commands() {
        let sessions = vec![
            create_test_session("/proj", vec!["cargo test", "cargo build"], vec![], 10),
            create_test_session("/proj", vec!["cargo test"], vec![], 5),
            create_test_session("/proj", vec!["npm test"], vec![], 8),
        ];

        let result = ProjectAnalyzer::analyze_by_project(&sessions);
        let stats = &result[0];

        assert_eq!(stats.top_commands[0].command, "cargo test");
        assert_eq!(stats.top_commands[0].count, 2);
    }

    #[test]
    fn test_empty_sessions() {
        let sessions = vec![];
        let result = ProjectAnalyzer::analyze_by_project(&sessions);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_session_without_stats() {
        let sessions = vec![SessionMeta {
            provider_id: "claude".to_string(),
            session_id: "test".to_string(),
            project_dir: Some("/proj".to_string()),
            stats: None,
            created_at: Some(chrono::Utc::now().timestamp_millis()),
            last_active_at: Some(chrono::Utc::now().timestamp_millis()),
            title: None,
            summary: None,
            source_path: None,
            resume_command: None,
        }];

        let result = ProjectAnalyzer::analyze_by_project(&sessions);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].session_count, 1);
        assert_eq!(result[0].top_commands.len(), 0);
    }
}
