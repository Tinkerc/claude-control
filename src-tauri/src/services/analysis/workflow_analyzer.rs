use std::collections::HashMap;
use crate::session_manager::SessionMeta;
use serde::{Serialize, Deserialize};

/// Workflow pattern detection analyzer
pub struct WorkflowAnalyzer;

// Re-use ToolFrequency from project_analyzer
use super::project_analyzer::ToolFrequency;

impl WorkflowAnalyzer {
    /// Detect workflow patterns across sessions
    pub fn detect_patterns(sessions: &[SessionMeta]) -> WorkflowPatterns {
        let command_sequences = Self::detect_sequences(sessions);
        let tool_usage = Self::analyze_tool_usage(sessions);
        let length_dist = Self::analyze_session_lengths(sessions);
        let common_flows = Self::detect_tool_sequences(sessions);

        WorkflowPatterns {
            command_sequences,
            tool_usage_distribution: tool_usage,
            session_length_distribution: length_dist,
            most_common_sequences: common_flows,
        }
    }

    fn detect_sequences(sessions: &[SessionMeta]) -> Vec<CommandSequence> {
        let mut sequence_counts: HashMap<Vec<String>, usize> = HashMap::new();
        let mut session_examples: HashMap<Vec<String>, String> = HashMap::new();

        for session in sessions {
            if let Some(stats) = &session.stats {
                if let Some(cmds) = &stats.commands_executed {
                    // Extract sequences of 2-3 commands
                    for window in cmds.windows(2).chain(cmds.windows(3)) {
                        let seq: Vec<String> = window.to_vec();
                        *sequence_counts.entry(seq.clone()).or_insert(0) += 1;
                        session_examples.entry(seq).or_insert_with(|| session.session_id.clone());
                    }
                }
            }
        }

        // Filter to sequences that appear 3+ times
        sequence_counts
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|(seq, count)| {
                let example_id = session_examples[&seq].clone();
                CommandSequence {
                    sequence: seq,
                    occurrence_count: count,
                    example_session_id: example_id,
                }
            })
            .collect()
    }

    fn detect_tool_sequences(sessions: &[SessionMeta]) -> Vec<SequencePattern> {
        let mut flow_counts: HashMap<Vec<String>, usize> = HashMap::new();

        for session in sessions {
            if let Some(stats) = &session.stats {
                if let Some(tools) = &stats.tools_used {
                    for window in tools.windows(2).chain(tools.windows(3)) {
                        *flow_counts.entry(window.to_vec()).or_insert(0) += 1;
                    }
                }
            }
        }

        flow_counts
            .into_iter()
            .filter(|(_, count)| *count >= 5)
            .map(|(flow, count)| SequencePattern {
                pattern: flow.join(" → "),
                occurrence_count: count,
                pattern_type: "tool_flow".to_string(),
            })
            .collect()
    }

    fn analyze_tool_usage(sessions: &[SessionMeta]) -> Vec<ToolFrequency> {
        let mut tool_counts: HashMap<String, usize> = HashMap::new();
        let mut total = 0usize;

        for session in sessions {
            if let Some(stats) = &session.stats {
                if let Some(tools) = &stats.tools_used {
                    for tool in tools {
                        *tool_counts.entry(tool.clone()).or_insert(0) += 1;
                        total += 1;
                    }
                }
            }
        }

        let mut items: Vec<_> = tool_counts
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
        items
    }

    fn analyze_session_lengths(sessions: &[SessionMeta]) -> LengthDistribution {
        let mut quick = 0usize;
        let mut medium = 0usize;
        let mut deep = 0usize;

        for session in sessions {
            let duration = session.stats.as_ref()
                .and_then(|s| s.duration_minutes)
                .unwrap_or(0);

            if duration < 10 {
                quick += 1;
            } else if duration < 60 {
                medium += 1;
            } else {
                deep += 1;
            }
        }

        let total = quick + medium + deep;

        LengthDistribution {
            quick_count: quick,
            quick_percent: if total > 0 { (quick as f64 / total as f64) * 100.0 } else { 0.0 },
            medium_count: medium,
            medium_percent: if total > 0 { (medium as f64 / total as f64) * 100.0 } else { 0.0 },
            deep_count: deep,
            deep_percent: if total > 0 { (deep as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }
}

/// Workflow patterns across sessions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkflowPatterns {
    pub command_sequences: Vec<CommandSequence>,
    pub tool_usage_distribution: Vec<ToolFrequency>,
    pub session_length_distribution: LengthDistribution,
    pub most_common_sequences: Vec<SequencePattern>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommandSequence {
    pub sequence: Vec<String>,
    pub occurrence_count: usize,
    pub example_session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SequencePattern {
    pub pattern: String,
    pub occurrence_count: usize,
    pub pattern_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LengthDistribution {
    pub quick_count: usize,
    pub quick_percent: f64,
    pub medium_count: usize,
    pub medium_percent: f64,
    pub deep_count: usize,
    pub deep_percent: f64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_manager::{SessionMeta, SessionStats};

    fn create_test_session(
        commands: Vec<&str>,
        tools: Vec<&str>,
    ) -> SessionMeta {
        SessionMeta {
            provider_id: "claude".to_string(),
            session_id: "test-id".to_string(),
            project_dir: Some("/proj".to_string()),
            stats: Some(SessionStats {
                commands_executed: Some(commands.into_iter().map(String::from).collect()),
                tools_used: Some(tools.into_iter().map(String::from).collect()),
                duration_minutes: Some(10),
                files_modified: None,
                message_count: None,
                user_message_count: None,
                assistant_message_count: None,
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
    fn test_detect_command_sequences() {
        let sessions = vec![
            create_test_session(
                vec!["git add", "git commit", "git push"],
                vec![],
            ),
            create_test_session(
                vec!["git add", "git commit", "git push"],
                vec![],
            ),
            create_test_session(
                vec!["git add", "git commit", "git push"],
                vec![],
            ),
            create_test_session(
                vec!["cargo build", "cargo test"],
                vec![],
            ),
        ];

        let patterns = WorkflowAnalyzer::detect_patterns(&sessions);

        let git_seq = patterns.command_sequences.iter()
            .find(|p| p.sequence == vec!["git add", "git commit"]);
        assert!(git_seq.is_some());
        assert_eq!(git_seq.unwrap().occurrence_count, 3);
    }

    #[test]
    fn test_tool_usage_distribution() {
        let sessions = vec![
            create_test_session(vec![], vec!["Write", "Read"]),
            create_test_session(vec![], vec!["Write", "Bash"]),
            create_test_session(vec![], vec!["Write", "Read"]),
        ];

        let patterns = WorkflowAnalyzer::detect_patterns(&sessions);

        assert_eq!(patterns.tool_usage_distribution[0].tool, "Write");
        assert_eq!(patterns.tool_usage_distribution[0].usage_count, 3);
    }

    #[test]
    fn test_session_length_distribution() {
        let sessions = vec![
            create_test_session_with_duration(5),
            create_test_session_with_duration(15),
            create_test_session_with_duration(70),
        ];

        let patterns = WorkflowAnalyzer::detect_patterns(&sessions);

        assert_eq!(patterns.session_length_distribution.quick_count, 1);
        assert_eq!(patterns.session_length_distribution.medium_count, 1);
        assert_eq!(patterns.session_length_distribution.deep_count, 1);
    }

    fn create_test_session_with_duration(duration: u64) -> SessionMeta {
        SessionMeta {
            provider_id: "claude".to_string(),
            session_id: "test-id".to_string(),
            project_dir: Some("/proj".to_string()),
            stats: Some(SessionStats {
                commands_executed: None,
                tools_used: None,
                duration_minutes: Some(duration),
                files_modified: None,
                message_count: None,
                user_message_count: None,
                assistant_message_count: None,
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
}
