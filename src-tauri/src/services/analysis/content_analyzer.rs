use std::collections::HashMap;
use crate::session_manager::SessionMeta;
use serde::{Serialize, Deserialize};

/// Content analysis for task classification and concept extraction
pub struct ContentAnalyzer;

impl ContentAnalyzer {
    /// Analyze content across all sessions
    pub fn analyze(sessions: &[SessionMeta]) -> ContentAnalysis {
        let task_classification = Self::classify_tasks(sessions);
        let concepts = Self::extract_concepts(sessions);
        let prompt_patterns = Self::detect_prompt_patterns(sessions);

        ContentAnalysis {
            task_classification,
            concepts,
            prompt_patterns,
            total_sessions_analyzed: sessions.len(),
        }
    }

    /// Classify sessions by task type using keyword matching
    pub fn classify_tasks(sessions: &[SessionMeta]) -> Vec<TaskCategory> {
        let mut classifications: HashMap<String, usize> = HashMap::new();

        let patterns = vec![
            ("bug_fix", vec!["fix", "bug", "error", "broken", "doesn't work", "debug"]),
            ("feature", vec!["add", "implement", "create", "new", "feature"]),
            ("refactor", vec!["refactor", "rewrite", "clean up", "reorganize", "improve"]),
            ("documentation", vec!["document", "readme", "comment", "docstring"]),
            ("testing", vec!["test", "spec", "coverage", "mock"]),
            ("review", vec!["review", "check", "audit", "verify"]),
        ];

        for session in sessions {
            let first_prompt = session.stats.as_ref()
                .and_then(|s| s.topic.as_ref())
                .map(|t| t.to_lowercase())
                .unwrap_or_default();

            let mut classified = false;
            for (category, keywords) in &patterns {
                if keywords.iter().any(|kw| first_prompt.contains(kw)) {
                    *classifications.entry(category.to_string()).or_insert(0) += 1;
                    classified = true;
                    break;
                }
            }
            if !classified {
                *classifications.entry("other".to_string()).or_insert(0) += 1;
            }
        }

        classifications
            .into_iter()
            .map(|(category, count)| TaskCategory { category, count })
            .collect()
    }

    /// Extract technical concepts from session topics
    pub fn extract_concepts(sessions: &[SessionMeta]) -> Vec<ConceptEntry> {
        let mut word_counts: HashMap<String, usize> = HashMap::new();

        let tech_terms = [
            "api", "auth", "authentication", "database", "schema", "endpoint",
            "frontend", "backend", "component", "service", "module",
            "rust", "typescript", "react", "sql", "redis",
            "docker", "kubernetes", "test", "deploy", "build",
            "config", "settings", "env", "production", "development",
        ];

        for session in sessions {
            let prompt = session.stats.as_ref()
                .and_then(|s| s.topic.as_ref())
                .unwrap_or(&String::new())
                .to_lowercase();

            for term in &tech_terms {
                if prompt.contains(term) {
                    *word_counts.entry(term.to_string()).or_insert(0) += 1;
                }
            }
        }

        word_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(concept, mentions)| ConceptEntry { concept, mentions })
            .collect()
    }

    /// Detect repeated prompt patterns
    pub fn detect_prompt_patterns(sessions: &[SessionMeta]) -> Vec<PromptPattern> {
        let mut patterns: HashMap<String, usize> = HashMap::new();

        for session in sessions {
            let prompt = session.stats.as_ref()
                .and_then(|s| s.topic.as_ref())
                .unwrap_or(&String::new())
                .to_lowercase();

            // Simple pattern detection
            if prompt.starts_with("add ") && prompt.contains(" test") {
                *patterns.entry("add tests for".to_string()).or_insert(0) += 1;
            }
            if prompt.starts_with("implement ") {
                *patterns.entry("implement feature".to_string()).or_insert(0) += 1;
            }
            if prompt.starts_with("refactor ") {
                *patterns.entry("refactor code".to_string()).or_insert(0) += 1;
            }
            if prompt.starts_with("fix ") || prompt.contains("bug") {
                *patterns.entry("fix bug".to_string()).or_insert(0) += 1;
            }
        }

        patterns
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|(pattern, count)| PromptPattern {
                pattern,
                occurrence_count: count,
                example_usage: None,
            })
            .collect()
    }
}

/// Content analysis results
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentAnalysis {
    pub task_classification: Vec<TaskCategory>,
    pub concepts: Vec<ConceptEntry>,
    pub prompt_patterns: Vec<PromptPattern>,
    pub total_sessions_analyzed: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TaskCategory {
    pub category: String,
    pub count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConceptEntry {
    pub concept: String,
    pub mentions: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PromptPattern {
    pub pattern: String,
    pub occurrence_count: usize,
    pub example_usage: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_manager::{SessionMeta, SessionStats};

    fn create_session_with_topic(topic: &str) -> SessionMeta {
        SessionMeta {
            provider_id: "claude".to_string(),
            session_id: "test".to_string(),
            project_dir: Some("/proj".to_string()),
            stats: Some(SessionStats {
                topic: Some(topic.to_string()),
                commands_executed: None,
                tools_used: None,
                duration_minutes: None,
                files_modified: None,
                message_count: None,
                user_message_count: None,
                assistant_message_count: None,
                token_usage: None,
                model: None,
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
    fn test_classify_bug_fix_tasks() {
        let sessions = vec![
            create_session_with_topic("Fix the login bug"),
            create_session_with_topic("Fix authentication error"),
            create_session_with_topic("Add new feature"),
        ];

        let classifications = ContentAnalyzer::classify_tasks(&sessions);

        let bug_fix = classifications.iter().find(|c| c.category == "bug_fix");
        assert!(bug_fix.is_some());
        assert_eq!(bug_fix.unwrap().count, 2);
    }

    #[test]
    fn test_extract_concepts() {
        let sessions = vec![
            create_session_with_topic("Add authentication to the API"),
            create_session_with_topic("Fix database schema for auth"),
        ];

        let concepts = ContentAnalyzer::extract_concepts(&sessions);

        let auth = concepts.iter().find(|c| c.concept == "auth");
        assert!(auth.is_some());
    }

    #[test]
    fn test_empty_sessions() {
        let sessions = vec![];
        let analysis = ContentAnalyzer::analyze(&sessions);
        assert_eq!(analysis.total_sessions_analyzed, 0);
        assert_eq!(analysis.task_classification.len(), 0);
    }
}
