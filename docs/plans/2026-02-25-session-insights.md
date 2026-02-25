# Session Insights & Pattern Detection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an intelligent session analysis system that learns from Claude Code session history and provides actionable insights about work patterns, project-specific behaviors, and content trends.

**Architecture:** Rule-based analysis engine in Rust (backend) + React components (frontend) that aggregates session data to detect patterns in commands, files, tools, and content. No new database tables - uses existing session data with in-memory analysis.

**Tech Stack:** Rust (Tauri), TypeScript (React), SQLite (existing), serde (serialization)

---

## Backend: Analysis Service Module

### Task 1: Create analysis module structure

**Files:**
- Create: `src-tauri/src/services/analysis/mod.rs`
- Modify: `src-tauri/src/services/mod.rs`

**Step 1: Create the analysis module file**

```rust
// src-tauri/src/services/analysis/mod.rs

//! Session insights and pattern detection analysis services.
//!
//! Provides rule-based analysis of Claude Code sessions to extract:
//! - Project-level statistics (commands, files, tools)
//! - Workflow patterns (command sequences, tool flows)
//! - Content analysis (task types, concepts, prompt patterns)
//! - Session similarity matching

pub mod project_analyzer;
pub mod workflow_analyzer;
pub mod content_analyzer;
pub mod similarity_finder;

pub use project_analyzer::{ProjectAnalyzer, ProjectStats, CommandFrequency, FileFrequency, ToolFrequency};
pub use workflow_analyzer::{WorkflowAnalyzer, WorkflowPatterns, CommandSequence, SequencePattern, LengthDistribution};
pub use content_analyzer::{ContentAnalyzer, ContentAnalysis, TaskCategory, ConceptEntry, PromptPattern};
pub use similarity_finder::{SimilarityFinder, SimilarSession};

/// Combined insights for a project or all projects
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AllInsights {
    pub project_stats: Vec<ProjectStats>,
    pub workflow_patterns: WorkflowPatterns,
    pub content_analysis: ContentAnalysis,
    pub total_sessions_analyzed: usize,
}

/// Errors that can occur during analysis
#[derive(Debug)]
pub enum AnalysisError {
    NoData(String),
    TooManyCorruptedSessions(String),
    AnalysisFailed(String),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnalysisError::NoData(msg) => write!(f, "No data: {}", msg),
            AnalysisError::TooManyCorruptedSessions(msg) => write!(f, "Data corruption: {}", msg),
            AnalysisError::AnalysisFailed(msg) => write!(f, "Analysis failed: {}", msg),
        }
    }
}

impl std::error::Error for AnalysisError {}
```

**Step 2: Update services module to export analysis**

```rust
// src-tauri/src/services/mod.rs

// Add this line after other pub use statements
pub mod analysis;

pub use analysis::{
    AllInsights, ProjectStats, WorkflowPatterns, ContentAnalysis, SimilarSession,
    // ... other exports
};
```

**Step 3: Commit**

```bash
git add src-tauri/src/services/analysis/mod.rs src-tauri/src/services/mod.rs
git commit -m "feat(analysis): create analysis module structure"
```

---

### Task 2: Implement ProjectAnalyzer

**Files:**
- Create: `src-tauri/src/services/analysis/project_analyzer.rs`
- Test: `src-tauri/src/services/analysis/project_analyzer_tests.rs`

**Step 1: Write the failing tests first**

```rust
// src-tauri/src/services/analysis/project_analyzer_tests.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_manager::{SessionMeta, SessionStats};

    fn create_test_session(
        project: &str,
        commands: Vec<&str>,
        files: Vec<&str>,
        duration: i64,
    ) -> SessionMeta {
        SessionMeta {
            id: "test-id".to_string(),
            project_path: project.to_string(),
            stats: Some(SessionStats {
                commands_executed: commands.into_iter().map(String::from).collect(),
                files_modified: files.into_iter().map(String::from).collect(),
                duration_minutes: duration,
                tools_used: vec!["Write".to_string(), "Read".to_string()],
                message_count: 10,
                user_message_count: 5,
                assistant_message_count: 5,
                token_usage: None,
                model: None,
                topic: None,
            }),
            created_at: chrono::Utc::now(),
            last_active_at: chrono::Utc::now(),
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
        assert_eq!(result[0].project_path, "/proj/a");
        assert_eq!(result[0].session_count, 2);
        assert_eq!(result[1].project_path, "/proj/b");
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
            id: "test".to_string(),
            project_path: "/proj".to_string(),
            stats: None,
            created_at: chrono::Utc::now(),
            last_active_at: chrono::Utc::now(),
        }];

        let result = ProjectAnalyzer::analyze_by_project(&sessions);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].session_count, 1);
        assert_eq!(result[0].top_commands.len(), 0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib project_analyzer`
Expected: FAIL with "module not found" or "undefined symbol"

**Step 3: Implement ProjectAnalyzer**

```rust
// src-tauri/src/services/analysis/project_analyzer.rs

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
            project_map
                .entry(session.project_path.clone())
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
        let mut total_duration = 0i64;

        for session in sessions {
            if let Some(stats) = &session.stats {
                total_duration += stats.duration_minutes;

                for cmd in &stats.commands_executed {
                    *cmd_counts.entry(cmd.clone()).or_insert(0) += 1;
                }
                for file in &stats.files_modified {
                    *file_counts.entry(file.clone()).or_insert(0) += 1;
                }
                for tool in &stats.tools_used {
                    *tool_counts.entry(tool.clone()).or_insert(0) += 1;
                }
            }
        }

        let total_cmds: usize = cmd_counts.values().sum();
        let total_files: usize = file_counts.values().sum();
        let total_tools: usize = tool_counts.values().sum();

        ProjectStats {
            project_path: path.to_string(),
            session_count: sessions.len(),
            total_duration_minutes: total_duration,
            top_commands: Self::top_n(cmd_counts, total_cmds, 10),
            top_files: Self::top_n(file_counts, total_files, 10),
            top_tools: Self::top_n(tool_counts, total_tools, 10),
            avg_session_duration: if sessions.is_empty() {
                0.0
            } else {
                total_duration as f64 / sessions.len() as f64
            },
        }
    }

    fn top_n(counts: HashMap<String, usize>, total: usize, n: usize) -> Vec<FrequencyItem> {
        let mut items: Vec<_> = counts
            .into_iter()
            .map(|(name, count)| FrequencyItem {
                name,
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
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ProjectStats {
    pub project_path: String,
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

pub type FileFrequency = FrequencyItem;
pub type ToolFrequency = FrequencyItem;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct FrequencyItem {
    name: String,
    count: usize,
    percentage: f64,
}

// Convert FrequencyItem to the public types
impl From<FrequencyItem> for FileFrequency {
    fn from(item: FrequencyItem) -> Self {
        FileFrequency {
            file: item.name,
            edit_count: item.count,
            percentage: item.percentage,
        }
    }
}

impl From<FrequencyItem> for ToolFrequency {
    fn from(item: FrequencyItem) -> Self {
        ToolFrequency {
            tool: item.name,
            usage_count: item.count,
            percentage: item.percentage,
        }
    }
}

// Add the proper structs
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib project_analyzer`
Expected: PASS

**Step 5: Add tests to module**

```rust
// src-tauri/src/services/analysis/project_analyzer.rs

// Add at the bottom of the file
#[cfg(test)]
mod tests {
    use super::*;

    // ... paste the test functions from Step 1 ...
}
```

**Step 6: Run tests again to verify**

Run: `cargo test --lib project_analyzer`
Expected: PASS

**Step 7: Commit**

```bash
git add src-tauri/src/services/analysis/project_analyzer.rs
git commit -m "feat(analysis): implement ProjectAnalyzer with tests"
```

---

### Task 3: Implement WorkflowAnalyzer

**Files:**
- Create: `src-tauri/src/services/analysis/workflow_analyzer.rs`

**Step 1: Write the failing test**

```rust
// Add to workflow_analyzer.rs at the bottom

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session(
        commands: Vec<&str>,
        tools: Vec<&str>,
    ) -> SessionMeta {
        SessionMeta {
            id: "test-id".to_string(),
            project_path: "/proj".to_string(),
            stats: Some(SessionStats {
                commands_executed: commands.into_iter().map(String::from).collect(),
                tools_used: tools.into_iter().map(String::from).collect(),
                duration_minutes: 10,
                ..Default::default()
            }),
            ..Default::default()
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
                vec!["cargo build", "cargo test"],
                vec![],
            ),
        ];

        let patterns = WorkflowAnalyzer::detect_sequences(&sessions);

        let git_seq = patterns.iter()
            .find(|p| p.sequence == vec!["git add", "git commit"]);
        assert!(git_seq.is_some());
        assert_eq!(git_seq.unwrap().occurrence_count, 2);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib workflow_analyzer`
Expected: FAIL

**Step 3: Implement WorkflowAnalyzer**

```rust
// src-tauri/src/services/analysis/workflow_analyzer.rs

use std::collections::HashMap;
use crate::session_manager::SessionMeta;

/// Workflow pattern detection analyzer
pub struct WorkflowAnalyzer;

impl WorkflowAnalyzer {
    /// Detect command sequences across sessions
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
                let cmds = &stats.commands_executed;
                // Extract sequences of 2-3 commands
                for window in cmds.windows(2).chain(cmds.windows(3)) {
                    let seq: Vec<String> = window.to_vec();
                    *sequence_counts.entry(seq.clone()).or_insert(0) += 1;
                    session_examples.entry(seq).or_insert_with(|| session.id.clone());
                }
            }
        }

        // Filter to sequences that appear 3+ times
        sequence_counts
            .into_iter()
            .filter(|(_, count)| *count >= 3)
            .map(|(seq, count)| CommandSequence {
                sequence: seq,
                occurrence_count: count,
                example_session_id: session_examples[&seq].clone(),
            })
            .collect()
    }

    fn detect_tool_sequences(sessions: &[SessionMeta]) -> Vec<SequencePattern> {
        let mut flow_counts: HashMap<Vec<String>, usize> = HashMap::new();

        for session in sessions {
            if let Some(stats) = &session.stats {
                let tools = &stats.tools_used;
                for window in tools.windows(2).chain(tools.windows(3)) {
                    *flow_counts.entry(window.to_vec()).or_insert(0) += 1;
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
                for tool in &stats.tools_used {
                    *tool_counts.entry(tool.clone()).or_insert(0) += 1;
                    total += 1;
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
                .map(|s| s.duration_minutes)
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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct WorkflowPatterns {
    pub command_sequences: Vec<CommandSequence>,
    pub tool_usage_distribution: Vec<ToolFrequency>,
    pub session_length_distribution: LengthDistribution,
    pub most_common_sequences: Vec<SequencePattern>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CommandSequence {
    pub sequence: Vec<String>,
    pub occurrence_count: usize,
    pub example_session_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SequencePattern {
    pub pattern: String,
    pub occurrence_count: usize,
    pub pattern_type: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct LengthDistribution {
    pub quick_count: usize,
    pub quick_percent: f64,
    pub medium_count: usize,
    pub medium_percent: f64,
    pub deep_count: usize,
    pub deep_percent: f64,
}

// Re-use ToolFrequency from project_analyzer
use super::project_analyzer::ToolFrequency;
```

**Step 4: Run tests**

Run: `cargo test --lib workflow_analyzer`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/services/analysis/workflow_analyzer.rs
git commit -m "feat(analysis): implement WorkflowAnalyzer with pattern detection"
```

---

### Task 4: Implement ContentAnalyzer

**Files:**
- Create: `src-tauri/src/services/analysis/content_analyzer.rs`

**Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_session_with_topic(topic: &str) -> SessionMeta {
        SessionMeta {
            id: "test".to_string(),
            project_path: "/proj".to_string(),
            stats: Some(SessionStats {
                topic: Some(topic.to_string()),
                ..Default::default()
            }),
            ..Default::default()
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
}
```

**Step 2: Run tests to verify fail**

Run: `cargo test --lib content_analyzer`
Expected: FAIL

**Step 3: Implement ContentAnalyzer**

```rust
// src-tauri/src/services/analysis/content_analyzer.rs

use std::collections::HashMap;
use crate::session_manager::SessionMeta;

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

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ContentAnalysis {
    pub task_classification: Vec<TaskCategory>,
    pub concepts: Vec<ConceptEntry>,
    pub prompt_patterns: Vec<PromptPattern>,
    pub total_sessions_analyzed: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct TaskCategory {
    pub category: String,
    pub count: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ConceptEntry {
    pub concept: String,
    pub mentions: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct PromptPattern {
    pub pattern: String,
    pub occurrence_count: usize,
    pub example_usage: Option<String>,
}
```

**Step 4: Run tests**

Run: `cargo test --lib content_analyzer`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/services/analysis/content_analyzer.rs
git commit -m "feat(analysis): implement ContentAnalyzer for task classification"
```

---

### Task 5: Implement SimilarityFinder

**Files:**
- Create: `src-tauri/src/services/analysis/similarity_finder.rs`

**Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session(
        project: &str,
        commands: Vec<&str>,
        files: Vec<&str>,
        topic: &str,
    ) -> SessionMeta {
        SessionMeta {
            id: format!("id-{}", project),
            project_path: project.to_string(),
            stats: Some(SessionStats {
                commands_executed: commands.into_iter().map(String::from).collect(),
                files_modified: files.into_iter().map(String::from).collect(),
                topic: Some(topic.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn test_similarity_same_project() {
        let session_a = create_test_session("/proj/a", vec!["test"], vec!["a.rs"], "Fix bug");
        let session_b = create_test_session("/proj/a", vec!["build"], vec!["b.rs"], "Add feature");
        let session_c = create_test_session("/proj/b", vec!["test"], vec!["a.rs"], "Fix bug");

        let similar = SimilarityFinder::find_similar(&session_a, &[session_b, session_c], 10);

        assert_eq!(similar.len(), 2);
        // Same project should rank higher
        assert_eq!(similar[0].session_id, session_b.id);
        assert!(similar[0].similarity_score > 0.3);
    }

    #[test]
    fn test_similarity_shared_files() {
        let session_a = create_test_session("/proj", vec!["test"], vec!["src/auth.rs"], "Fix auth");
        let session_b = create_test_session("/proj", vec!["build"], vec!["src/auth.rs"], "Fix auth");
        let session_c = create_test_session("/proj", vec!["test"], vec!["src/test.rs"], "Add test");

        let similar = SimilarityFinder::find_similar(&session_a, &[session_b, session_c], 10);

        assert_eq!(similar[0].session_id, session_b.id);
        assert!(similar[0].similarity_reason.contains("shared files"));
    }
}
```

**Step 2: Run tests**

Run: `cargo test --lib similarity_finder`
Expected: FAIL

**Step 3: Implement SimilarityFinder**

```rust
// src-tauri/src/services/analysis/similarity_finder.rs

use std::collections::HashSet;
use crate::session_manager::SessionMeta;

/// Find similar sessions using multi-factor scoring
pub struct SimilarityFinder;

impl SimilarityFinder {
    /// Find sessions similar to the target session
    pub fn find_similar(
        target: &SessionMeta,
        all_sessions: &[SessionMeta],
        limit: usize,
    ) -> Vec<SimilarSession> {
        let mut scored: Vec<(SimilarSession, f64)> = Vec::new();

        for session in all_sessions {
            if session.id == target.id {
                continue;
            }

            let (score, reason) = Self::calculate_similarity(target, session);
            if score > 0.3 {
                scored.push((
                    SimilarSession {
                        session_id: session.id.clone(),
                        similarity_score: score,
                        similarity_reason: reason,
                        session: session.clone(),
                    },
                    score,
                ));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored.into_iter()
            .take(limit)
            .map(|(s, _)| s)
            .collect()
    }

    fn calculate_similarity(
        a: &SessionMeta,
        b: &SessionMeta,
    ) -> (f64, String) {
        let mut score = 0f64;
        let mut reasons = Vec::new();

        // Same project?
        if a.project_path == b.project_path {
            score += 0.3;
            reasons.push("same project".to_string());
        }

        // Overlapping files?
        if let (Some(a_stats), Some(b_stats)) = (&a.stats, &b.stats) {
            let overlap: HashSet<_> = a_stats.files_modified
                .iter()
                .filter(|f| b_stats.files_modified.contains(f))
                .collect();

            if !overlap.is_empty() {
                let overlap_ratio = overlap.len() as f64 /
                    a_stats.files_modified.len().max(1) as f64;
                score += overlap_ratio * 0.4;
                reasons.push(format!("{} shared files", overlap.len()));
            }

            // Similar task type?
            if let (Some(a_topic), Some(b_topic)) = (&a_stats.topic, &b_stats.topic) {
                if Self::task_type_matches(a_topic, b_topic) {
                    score += 0.2;
                    reasons.push("similar task type".to_string());
                }
            }

            // Similar commands?
            let cmd_overlap: HashSet<_> = a_stats.commands_executed
                .iter()
                .filter(|c| b_stats.commands_executed.contains(c))
                .collect();

            if !cmd_overlap.is_empty() {
                let cmd_ratio = cmd_overlap.len() as f64 /
                    a_stats.commands_executed.len().max(1) as f64;
                score += cmd_ratio * 0.1;
                reasons.push(format!("{} shared commands", cmd_overlap.len()));
            }
        }

        (score.min(1.0), reasons.join(", "))
    }

    fn task_type_matches(a: &str, b: &str) -> bool {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        let keywords = [
            "bug", "fix", "error", "debug",
            "add", "implement", "create", "new",
            "refactor", "clean", "improve",
        ];

        for kw in &keywords {
            if a_lower.contains(kw) && b_lower.contains(kw) {
                return true;
            }
        }
        false
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct SimilarSession {
    pub session_id: String,
    pub similarity_score: f64,
    pub similarity_reason: String,
    pub session: SessionMeta,
}
```

**Step 4: Run tests**

Run: `cargo test --lib similarity_finder`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/services/analysis/similarity_finder.rs
git commit -m "feat(analysis): implement SimilarityFinder for session matching"
```

---

## Backend: Tauri Commands

### Task 6: Create insights commands module

**Files:**
- Create: `src-tauri/src/commands/insights.rs`
- Modify: `src-tauri/src/commands/mod.rs`

**Step 1: Create commands module**

```rust
// src-tauri/src/commands/insights.rs

use crate::services::{
    analysis::{ProjectAnalyzer, WorkflowAnalyzer, ContentAnalyzer, SimilarityFinder},
    session_service::SessionService,
};
use tauri::State;

/// Get project statistics for all or specific project
#[tauri::command]
pub async fn get_project_stats(
    project_path: Option<String>,
    session_service: State<'_, SessionService>,
) -> Result<Vec<ProjectStats>, String> {
    let sessions = session_service.get_all_sessions()
        .await
        .map_err(|e| e.to_string())?;

    let filtered = if let Some(path) = project_path {
        sessions.into_iter()
            .filter(|s| s.project_path == path)
            .collect()
    } else {
        sessions
    };

    Ok(ProjectAnalyzer::analyze_by_project(&filtered))
}

/// Get workflow patterns for all or specific project
#[tauri::command]
pub async fn get_workflow_patterns(
    project_path: Option<String>,
    session_service: State<'_, SessionService>,
) -> Result<WorkflowPatterns, String> {
    let sessions = session_service.get_all_sessions()
        .await
        .map_err(|e| e.to_string())?;

    let filtered = if let Some(path) = project_path {
        sessions.into_iter()
            .filter(|s| s.project_path == path)
            .collect()
    } else {
        sessions
    };

    Ok(WorkflowAnalyzer::detect_patterns(&filtered))
}

/// Get content analysis for all or specific project
#[tauri::command]
pub async fn get_content_analysis(
    project_path: Option<String>,
    session_service: State<'_, SessionService>,
) -> Result<ContentAnalysis, String> {
    let sessions = session_service.get_all_sessions()
        .await
        .map_err(|e| e.to_string())?;

    let filtered = if let Some(path) = project_path {
        sessions.into_iter()
            .filter(|s| s.project_path == path)
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
    session_service: State<'_, SessionService>,
) -> Result<Vec<SimilarSession>, String> {
    let sessions = session_service.get_all_sessions()
        .await
        .map_err(|e| e.to_string())?;

    let target = sessions.iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    Ok(SimilarityFinder::find_similar(target, &sessions, limit.unwrap_or(5)))
}

/// Get all insights combined
#[tauri::command]
pub async fn get_all_insights(
    project_path: Option<String>,
    session_service: State<'_, SessionService>,
) -> Result<AllInsights, String> {
    let sessions = session_service.get_all_sessions()
        .await
        .map_err(|e| e.to_string())?;

    if sessions.is_empty() {
        return Err("No sessions available for analysis".to_string());
    }

    let filtered = if let Some(path) = project_path {
        sessions.into_iter()
            .filter(|s| s.project_path == path)
            .collect()
    } else {
        sessions
    };

    let total = filtered.len();

    Ok(AllInsights {
        project_stats: ProjectAnalyzer::analyze_by_project(&filtered),
        workflow_patterns: WorkflowAnalyzer::detect_patterns(&filtered),
        content_analysis: ContentAnalyzer::analyze(&filtered),
        total_sessions_analyzed: total,
    })
}
```

**Step 2: Update commands module**

```rust
// src-tauri/src/commands/mod.rs

// Add pub mod insights;
pub mod insights;

// And re-export types if needed
pub use insights::AllInsights;
```

**Step 3: Update lib.rs to register commands**

```rust
// src-tauri/src/lib.rs

// Add to invoke_handler in run() function
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    get_project_stats,
    get_workflow_patterns,
    get_content_analysis,
    find_similar_sessions,
    get_all_insights,
])
```

**Step 4: Commit**

```bash
git add src-tauri/src/commands/insights.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat(commands): add insights Tauri commands"
```

---

## Frontend: Type Definitions

### Task 7: Add TypeScript types for insights

**Files:**
- Modify: `src/types.ts`

**Step 1: Add insights types**

```typescript
// src/types.ts

// Add after existing session types

// Project Statistics
export interface ProjectStats {
  project_path: string;
  session_count: number;
  total_duration_minutes: number;
  top_commands: CommandFrequency[];
  top_files: FileFrequency[];
  top_tools: ToolFrequency[];
  avg_session_duration: number;
}

export interface CommandFrequency {
  command: string;
  count: number;
  percentage: number;
}

export interface FileFrequency {
  file: string;
  edit_count: number;
  percentage: number;
}

export interface ToolFrequency {
  tool: string;
  usage_count: number;
  percentage: number;
}

// Workflow Patterns
export interface WorkflowPatterns {
  command_sequences: CommandSequence[];
  tool_usage_distribution: ToolFrequency[];
  session_length_distribution: LengthDistribution;
  most_common_sequences: SequencePattern[];
}

export interface CommandSequence {
  sequence: string[];
  occurrence_count: number;
  example_session_id: string;
}

export interface SequencePattern {
  pattern: string;
  occurrence_count: number;
  pattern_type: string;
}

export interface LengthDistribution {
  quick_count: number;
  quick_percent: number;
  medium_count: number;
  medium_percent: number;
  deep_count: number;
  deep_percent: number;
}

// Content Analysis
export interface ContentAnalysis {
  task_classification: TaskCategory[];
  concepts: ConceptEntry[];
  prompt_patterns: PromptPattern[];
  total_sessions_analyzed: number;
}

export interface TaskCategory {
  category: string;
  count: number;
}

export interface ConceptEntry {
  concept: string;
  mentions: number;
}

export interface PromptPattern {
  pattern: string;
  occurrence_count: number;
  example_usage: string | null;
}

// Similar Sessions
export interface SimilarSession {
  session_id: string;
  similarity_score: number;
  similarity_reason: string;
  session: SessionMeta;
}

// Combined Insights
export interface AllInsights {
  project_stats: ProjectStats[];
  workflow_patterns: WorkflowPatterns;
  content_analysis: ContentAnalysis;
  total_sessions_analyzed: number;
}
```

**Step 2: Commit**

```bash
git add src/types.ts
git commit -m "feat(types): add insights TypeScript types"
```

---

## Frontend: Insights Components

### Task 8: Create InsightsTab component

**Files:**
- Create: `src/components/sessions/insights/InsightsTab.tsx`
- Create: `src/components/sessions/insights/index.ts`

**Step 1: Create the insights directory and main tab component**

```typescript
// src/components/sessions/insights/InsightsTab.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { AllInsights } from '@/types';
import { useTranslation } from 'react-i18next';

type ViewMode = 'overview' | 'projects' | 'workflow' | 'content';

type LoadingState = 'idle' | 'loading' | 'success' | 'error';

export function InsightsTab() {
  const { t } = useTranslation();
  const [viewMode, setViewMode] = useState<ViewMode>('overview');
  const [selectedProject, setSelectedProject] = useState<string | null>(null);
  const [projects, setProjects] = useState<string[]>([]);

  const [state, setState] = useState<{
    status: LoadingState;
    error: string | null;
    data: AllInsights | null;
  }>({
    status: 'idle',
    error: null,
    data: null,
  });

  // Extract unique projects from data
  useEffect(() => {
    if (state.data) {
      const uniqueProjects = Array.from(
        new Set(state.data.project_stats.map(p => p.project_path))
      );
      setProjects(uniqueProjects);
    }
  }, [state.data]);

  const loadInsights = async () => {
    setState({ status: 'loading', error: null, data: null });

    try {
      const data = await invoke<AllInsights>('get_all_insights', {
        projectPath: selectedProject,
      });

      if (data.total_sessions_analyzed === 0) {
        setState({
          status: 'error',
          error: t('insights.noData'),
          data: null,
        });
        return;
      }

      setState({ status: 'success', error: null, data });
    } catch (err) {
      setState({
        status: 'error',
        error: t('insights.loadError'),
        data: null,
      });
    }
  };

  useEffect(() => {
    loadInsights();
  }, [selectedProject]);

  // Empty state
  if (state.status === 'idle' || (state.status === 'success' && state.data?.total_sessions_analyzed === 0)) {
    return (
      <div className="insights-empty">
        <div className="empty-icon">📊</div>
        <h3>{t('insights.emptyTitle')}</h3>
        <p>{t('insights.emptyMessage')}</p>
      </div>
    );
  }

  // Error state
  if (state.status === 'error') {
    return (
      <div className="insights-error">
        <div className="error-icon">⚠️</div>
        <h3>{t('insights.errorTitle')}</h3>
        <p>{state.error}</p>
        <button onClick={loadInsights}>{t('common.retry')}</button>
      </div>
    );
  }

  // Loading state
  if (state.status === 'loading') {
    return (
      <div className="insights-loading">
        <div className="spinner" />
        <p>{t('insights.loading')}</p>
      </div>
    );
  }

  // Success state
  return (
    <div className="insights-tab">
      {/* Header with project selector */}
      <div className="insights-header">
        <h2>{t('insights.title')}</h2>
        {projects.length > 1 && (
          <select
            value={selectedProject || ''}
            onChange={(e) => setSelectedProject(e.target.value || null)}
          >
            <option value="">{t('insights.allProjects')}</option>
            {projects.map(p => (
              <option key={p} value={p}>{formatProjectName(p)}</option>
            ))}
          </select>
        )}
      </div>

      {/* View mode tabs */}
      <div className="insights-nav">
        <button
          className={viewMode === 'overview' ? 'active' : ''}
          onClick={() => setViewMode('overview')}
        >
          {t('insights.tabs.overview')}
        </button>
        <button
          className={viewMode === 'projects' ? 'active' : ''}
          onClick={() => setViewMode('projects')}
        >
          {t('insights.tabs.projects')}
        </button>
        <button
          className={viewMode === 'workflow' ? 'active' : ''}
          onClick={() => setViewMode('workflow')}
        >
          {t('insights.tabs.workflow')}
        </button>
        <button
          className={viewMode === 'content' ? 'active' : ''}
          onClick={() => setViewMode('content')}
        >
          {t('insights.tabs.content')}
        </button>
      </div>

      {/* Content */}
      <div className="insights-content">
        {viewMode === 'overview' && (
          <OverviewPanel insights={state.data!} />
        )}
        {viewMode === 'projects' && (
          <ProjectStatsPanel stats={state.data!.project_stats} />
        )}
        {viewMode === 'workflow' && (
          <WorkflowPatternsPanel patterns={state.data!.workflow_patterns} />
        )}
        {viewMode === 'content' && (
          <ContentAnalysisPanel analysis={state.data!.content_analysis} />
        )}
      </div>
    </div>
  );
}

function formatProjectName(path: string): string {
  const parts = path.split('/');
  return parts[parts.length - 1] || path;
}

// Placeholder panels (will implement in next tasks)
function OverviewPanel({ insights }: { insights: AllInsights }) {
  return <div className="overview-panel">Overview - Coming Soon</div>;
}

function ProjectStatsPanel({ stats }: { stats: AllInsights['project_stats'] }) {
  return <div className="project-stats-panel">Projects - Coming Soon</div>;
}

function WorkflowPatternsPanel({ patterns }: { patterns: WorkflowPatterns }) {
  return <div className="workflow-panel">Workflow - Coming Soon</div>;
}

function ContentAnalysisPanel({ analysis }: { analysis: ContentAnalysis }) {
  return <div className="content-panel">Content - Coming Soon</div>;
}
```

**Step 2: Create barrel export**

```typescript
// src/components/sessions/insights/index.ts

export { InsightsTab } from './InsightsTab';
```

**Step 3: Commit**

```bash
git add src/components/sessions/insights/
git commit -m "feat(frontend): create InsightsTab component with loading states"
```

---

### Task 9: Add Insights tab to SessionManagerPage

**Files:**
- Modify: `src/components/sessions/SessionManagerPage.tsx`

**Step 1: Update SessionManagerPage to include insights tab**

```typescript
// Find the tab state in SessionManagerPage and add insights option

// Add import
import { InsightsTab } from './insights';

// Update type if needed
type TabType = 'sessions' | 'insights';

// Update tab rendering
{activeTab === 'sessions' && (
  <SessionsList />
)}
{activeTab === 'insights' && (
  <InsightsTab />
)}
```

**Step 2: Add tab button in navigation**

```typescript
// Add tab button
<button
  className={activeTab === 'insights' ? 'active' : ''}
  onClick={() => setActiveTab('insights')}
>
  📊 {t('sessionManager.tabs.insights')}
</button>
```

**Step 3: Commit**

```bash
git add src/components/sessions/SessionManagerPage.tsx
git commit -m "feat(frontend): add Insights tab to SessionManagerPage"
```

---

### Task 10: Implement ProjectStatsPanel

**Files:**
- Create: `src/components/sessions/insights/ProjectStatsPanel.tsx`
- Create: `src/components/sessions/insights/CommandStatsChart.tsx`

**Step 1: Create CommandStatsChart component**

```typescript
// src/components/sessions/insights/CommandStatsChart.tsx

import type { CommandFrequency } from '@/types';

interface CommandStatsChartProps {
  commands: CommandFrequency[];
}

export function CommandStatsChart({ commands }: CommandStatsChartProps) {
  if (commands.length === 0) {
    return <div className="chart-empty">No commands data</div>;
  }

  const maxCount = Math.max(...commands.map(c => c.count));

  return (
    <div className="bar-chart">
      {commands.slice(0, 5).map((cmd) => (
        <div key={cmd.command} className="bar-row">
          <code className="bar-label">{cmd.command}</code>
          <div className="bar-container">
            <div
              className="bar-fill"
              style={{ width: `${(cmd.count / maxCount) * 100}%` }}
            />
            <span className="bar-value">{cmd.count}</span>
          </div>
        </div>
      ))}
    </div>
  );
}
```

**Step 2: Create ProjectStatsPanel**

```typescript
// src/components/sessions/insights/ProjectStatsPanel.tsx

import type { ProjectStats } from '@/types';
import { CommandStatsChart } from './CommandStatsChart';
import { useTranslation } from 'react-i18next';

interface ProjectStatsPanelProps {
  stats: ProjectStats[];
}

export function ProjectStatsPanel({ stats }: ProjectStatsPanelProps) {
  const { t } = useTranslation();

  if (stats.length === 0) {
    return <div className="panel-empty">No project data available</div>;
  }

  return (
    <div className="project-stats-panel">
      {stats.map((project) => (
        <div key={project.project_path} className="project-card">
          <h3>{formatProjectName(project.project_path)}</h3>

          {/* Summary metrics */}
          <div className="stats-summary">
            <div className="stat-item">
              <span className="stat-value">{project.session_count}</span>
              <span className="stat-label">{t('insights.sessions')}</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{formatDuration(project.total_duration_minutes)}</span>
              <span className="stat-label">{t('insights.totalTime')}</span>
            </div>
            <div className="stat-item">
              <span className="stat-value">{formatDuration(project.avg_session_duration)}</span>
              <span className="stat-label">{t('insights.avgDuration')}</span>
            </div>
          </div>

          {/* Top commands */}
          {project.top_commands.length > 0 && (
            <div className="stats-section">
              <h4>{t('insights.topCommands')}</h4>
              <CommandStatsChart commands={project.top_commands} />
            </div>
          )}

          {/* Top files */}
          {project.top_files.length > 0 && (
            <div className="stats-section">
              <h4>{t('insights.topFiles')}</h4>
              <FileStatsChart files={project.top_files} />
            </div>
          )}

          {/* Tool usage */}
          {project.top_tools.length > 0 && (
            <div className="stats-section">
              <h4>{t('insights.toolUsage')}</h4>
              <ToolUsagePieChart tools={project.top_tools} />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function formatProjectName(path: string): string {
  const parts = path.split('/');
  return parts[parts.length - 1] || path;
}

function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes}m`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}h ${mins}m` : `${hours}h`;
}

function FileStatsChart({ files }: { files: ProjectStats['top_files'] }) {
  const maxCount = Math.max(...files.map(f => f.edit_count), 1);

  return (
    <div className="bar-chart">
      {files.slice(0, 5).map((file) => (
        <div key={file.file} className="bar-row">
          <span className="bar-label">{formatFileName(file.file)}</span>
          <div className="bar-container">
            <div
              className="bar-fill bar-fill--files"
              style={{ width: `${(file.edit_count / maxCount) * 100}%` }}
            />
            <span className="bar-value">{file.edit_count}</span>
          </div>
        </div>
      ))}
    </div>
  );
}

function formatFileName(path: string): string {
  const parts = path.split('/');
  return parts[parts.length - 1] || path;
}

function ToolUsagePieChart({ tools }: { tools: ProjectStats['top_tools'] }) {
  return (
    <div className="tool-usage-chart">
      {tools.map((tool) => (
        <div key={tool.tool} className="tool-item">
          <span className="tool-name">{tool.tool}</span>
          <div className="tool-bar">
            <div
              className="tool-bar-fill"
              style={{ width: `${tool.percentage}%` }}
            />
            <span className="tool-percent">{Math.round(tool.percentage)}%</span>
          </div>
        </div>
      ))}
    </div>
  );
}
```

**Step 3: Update InsightsTab to use the real panel**

```typescript
// In src/components/sessions/insights/InsightsTab.tsx

// Update import
import { ProjectStatsPanel } from './ProjectStatsPanel';

// Replace placeholder
function ProjectStatsPanelWrapper({ stats }: { stats: AllInsights['project_stats'] }) {
  return <ProjectStatsPanel stats={stats} />;
}
```

**Step 4: Commit**

```bash
git add src/components/sessions/insights/
git commit -m "feat(frontend): implement ProjectStatsPanel with charts"
```

---

### Task 11: Implement WorkflowPatternsPanel

**Files:**
- Create: `src/components/sessions/insights/WorkflowPatternsPanel.tsx`

**Step 1: Create WorkflowPatternsPanel**

```typescript
// src/components/sessions/insights/WorkflowPatternsPanel.tsx

import type { WorkflowPatterns } from '@/types';
import { useTranslation } from 'react-i18next';

interface WorkflowPatternsPanelProps {
  patterns: WorkflowPatterns;
}

export function WorkflowPatternsPanel({ patterns }: WorkflowPatternsPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="workflow-panel">
      {/* Command sequences */}
      {patterns.command_sequences.length > 0 && (
        <div className="pattern-section">
          <h3>{t('insights.commonSequences')}</h3>
          <div className="sequence-flow">
            {patterns.command_sequences.slice(0, 5).map((seq, i) => (
              <div key={i} className="sequence-item">
                <div className="sequence-steps">
                  {seq.sequence.map((cmd, j) => (
                    <React.Fragment key={j}>
                      <code className="step-badge">{cmd}</code>
                      {j < seq.sequence.length - 1 && <span className="arrow">→</span>}
                    </React.Fragment>
                  ))}
                </div>
                <span className="sequence-count">{seq.occurrence_count}x</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Tool usage */}
      {patterns.tool_usage_distribution.length > 0 && (
        <div className="pattern-section">
          <h3>{t('insights.toolDistribution')}</h3>
          <div className="tool-distribution">
            {patterns.tool_usage_distribution.map((tool) => (
              <div key={tool.tool} className="tool-dist-item">
                <span className="tool-dist-name">{tool.tool}</span>
                <span className="tool-dist-percent">{Math.round(tool.percentage)}%</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Session length */}
      <div className="pattern-section">
        <h3>{t('insights.sessionLength')}</h3>
        <SessionLengthHistogram distribution={patterns.session_length_distribution} />
      </div>
    </div>
  );
}

function SessionLengthHistogram({ distribution }: { distribution: WorkflowPatterns['session_length_distribution'] }) {
  return (
    <div className="length-histogram">
      <div className="histogram-bar">
        <div className="bar-label">{t('insights.quickSessions')}</div>
        <div className="bar-track">
          <div
            className="bar-fill bar-fill--quick"
            style={{ width: `${distribution.quick_percent}%` }}
          />
        </div>
        <span className="bar-value">{distribution.quick_count}</span>
      </div>
      <div className="histogram-bar">
        <div className="bar-label">{t('insights.mediumSessions')}</div>
        <div className="bar-track">
          <div
            className="bar-fill bar-fill--medium"
            style={{ width: `${distribution.medium_percent}%` }}
          />
        </div>
        <span className="bar-value">{distribution.medium_count}</span>
      </div>
      <div className="histogram-bar">
        <div className="bar-label">{t('insights.deepSessions')}</div>
        <div className="bar-track">
          <div
            className="bar-fill bar-fill--deep"
            style={{ width: `${distribution.deep_percent}%` }}
          />
        </div>
        <span className="bar-value">{distribution.deep_count}</span>
      </div>
    </div>
  );
}
```

**Step 2: Update InsightsTab**

```typescript
// In src/components/sessions/insights/InsightsTab.tsx

import { WorkflowPatternsPanel } from './WorkflowPatternsPanel';

// Replace placeholder
function WorkflowPatternsPanelWrapper({ patterns }: { patterns: WorkflowPatterns }) {
  return <WorkflowPatternsPanel patterns={patterns} />;
}
```

**Step 3: Commit**

```bash
git add src/components/sessions/insights/
git commit -m "feat(frontend): implement WorkflowPatternsPanel"
```

---

### Task 12: Implement ContentAnalysisPanel

**Files:**
- Create: `src/components/sessions/insights/ContentAnalysisPanel.tsx`

**Step 1: Create ContentAnalysisPanel**

```typescript
// src/components/sessions/insights/ContentAnalysisPanel.tsx

import type { ContentAnalysis } from '@/types';
import { useTranslation } from 'react-i18next';

interface ContentAnalysisPanelProps {
  analysis: ContentAnalysis;
}

const TASK_COLORS: Record<string, string> = {
  bug_fix: 'bg-red-100 text-red-800',
  feature: 'bg-green-100 text-green-800',
  refactor: 'bg-blue-100 text-blue-800',
  documentation: 'bg-yellow-100 text-yellow-800',
  testing: 'bg-purple-100 text-purple-800',
  review: 'bg-orange-100 text-orange-800',
  other: 'bg-gray-100 text-gray-800',
};

export function ContentAnalysisPanel({ analysis }: ContentAnalysisPanelProps) {
  const { t } = useTranslation();

  return (
    <div className="content-panel">
      {/* Task classification */}
      {analysis.task_classification.length > 0 && (
        <div className="analysis-section">
          <h3>{t('insights.taskTypes')}</h3>
          <TaskTypeBadges tasks={analysis.task_classification} />
        </div>
      )}

      {/* Concept cloud */}
      {analysis.concepts.length > 0 && (
        <div className="analysis-section">
          <h3>{t('insights.keyConcepts')}</h3>
          <ConceptCloud concepts={analysis.concepts} />
        </div>
      )}

      {/* Prompt patterns */}
      {analysis.prompt_patterns.length > 0 && (
        <div className="analysis-section">
          <h3>{t('insights.promptPatterns')}</h3>
          <PromptPatternList patterns={analysis.prompt_patterns} />
        </div>
      )}

      <p className="analysis-summary">
        {t('insights.sessionsAnalyzed', { count: analysis.total_sessions_analyzed })}
      </p>
    </div>
  );
}

function TaskTypeBadges({ tasks }: { tasks: ContentAnalysis['task_classification'] }) {
  const total = tasks.reduce((sum, t) => sum + t.count, 0);

  return (
    <div className="task-badges">
      {tasks.map((task) => (
        <div
          key={task.category}
          className={`task-badge ${TASK_COLORS[task.category] || TASK_COLORS.other}`}
        >
          <span className="task-name">{formatTaskName(task.category)}</span>
          <span className="task-count">{task.count}</span>
          <span className="task-percent">{Math.round((task.count / total) * 100)}%</span>
        </div>
      ))}
    </div>
  );
}

function formatTaskName(category: string): string {
  const names: Record<string, string> = {
    bug_fix: 'Bug Fix',
    feature: 'Feature',
    refactor: 'Refactor',
    documentation: 'Docs',
    testing: 'Testing',
    review: 'Review',
    other: 'Other',
  };
  return names[category] || category;
}

function ConceptCloud({ concepts }: { concepts: ContentAnalysis['concepts'] }) {
  const maxMentions = Math.max(...concepts.map(c => c.mentions), 1);

  return (
    <div className="concept-cloud">
      {concepts.map((concept) => {
        const size = 0.8 + (concept.mentions / maxMentions) * 1.5;
        return (
          <span
            key={concept.concept}
            className="concept-tag"
            style={{ fontSize: `${size}rem` }}
          >
            {concept.concept}
            <span className="concept-count">{concept.mentions}</span>
          </span>
        );
      })}
    </div>
  );
}

function PromptPatternList({ patterns }: { patterns: ContentAnalysis['prompt_patterns'] }) {
  return (
    <div className="pattern-list">
      {patterns.map((pattern, i) => (
        <div key={i} className="pattern-item">
          <code className="pattern-text">"{pattern.pattern}"</code>
          <span className="pattern-count">{pattern.occurrence_count}x</span>
        </div>
      ))}
    </div>
  );
}
```

**Step 2: Update InsightsTab**

```typescript
// In src/components/sessions/insights/InsightsTab.tsx

import { ContentAnalysisPanel } from './ContentAnalysisPanel';

// Replace placeholder
function ContentAnalysisPanelWrapper({ analysis }: { analysis: ContentAnalysis }) {
  return <ContentAnalysisPanel analysis={analysis} />;
}
```

**Step 3: Commit**

```bash
git add src/components/sessions/insights/
git commit -m "feat(frontend): implement ContentAnalysisPanel"
```

---

### Task 13: Implement OverviewPanel

**Files:**
- Modify: `src/components/sessions/insights/InsightsTab.tsx`

**Step 1: Replace OverviewPanel placeholder with real implementation**

```typescript
// In src/components/sessions/insights/InsightsTab.tsx

function OverviewPanel({ insights }: { insights: AllInsights }) {
  const { t } = useTranslation();

  const topProject = insights.project_stats[0];
  const topTask = insights.content_analysis.task_classification[0];
  const topConcept = insights.content_analysis.concepts[0];

  return (
    <div className="overview-panel">
      <h2>{t('insights.overviewTitle')}</h2>

      {/* Summary cards */}
      <div className="overview-cards">
        <div className="overview-card">
          <div className="card-icon">📁</div>
          <div className="card-content">
            <div className="card-value">{insights.project_stats.length}</div>
            <div className="card-label">{t('insights.totalProjects')}</div>
          </div>
        </div>

        <div className="overview-card">
          <div className="card-icon">💬</div>
          <div className="card-content">
            <div className="card-value">{insights.total_sessions_analyzed}</div>
            <div className="card-label">{t('insights.totalSessions')}</div>
          </div>
        </div>

        {topTask && (
          <div className="overview-card">
            <div className="card-icon">🏷️</div>
            <div className="card-content">
              <div className="card-value">{formatTaskName(topTask.category)}</div>
              <div className="card-label">{t('insights.topTaskType')}</div>
            </div>
          </div>
        )}

        {topConcept && (
          <div className="overview-card">
            <div className="card-icon">🔑</div>
            <div className="card-content">
              <div className="card-value">{topConcept.concept}</div>
              <div className="card-label">{t('insights.topConcept')}</div>
            </div>
          </div>
        )}
      </div>

      {/* Most active project */}
      {topProject && (
        <div className="overview-section">
          <h3>{t('insights.mostActiveProject')}</h3>
          <div className="project-highlight">
            <span className="project-name">{formatProjectName(topProject.project_path)}</span>
            <span className="project-stats">
              {topProject.session_count} {t('insights.sessions').toLowerCase()} •
              {formatDuration(topProject.total_duration_minutes)}
            </span>
          </div>
        </div>
      )}

      {/* Top command sequence */}
      {insights.workflow_patterns.command_sequences.length > 0 && (
        <div className="overview-section">
          <h3>{t('insights.topCommandSequence')}</h3>
          <div className="sequence-highlight">
            {insights.workflow_patterns.command_sequences[0].sequence.join(' → ')}
            <span className="sequence-count">
              {insights.workflow_patterns.command_sequences[0].occurrence_count}x
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

function formatTaskName(category: string): string {
  const names: Record<string, string> = {
    bug_fix: 'Bug Fix',
    feature: 'Feature',
    refactor: 'Refactor',
    documentation: 'Docs',
    testing: 'Testing',
    review: 'Review',
    other: 'Other',
  };
  return names[category] || category;
}
```

**Step 2: Commit**

```bash
git add src/components/sessions/insights/InsightsTab.tsx
git commit -m "feat(frontend): implement OverviewPanel with summary cards"
```

---

### Task 14: Add SimilarSessionsModal

**Files:**
- Create: `src/components/sessions/SimilarSessionsModal.tsx`
- Modify: `src/components/sessions/SessionItem.tsx`

**Step 1: Create SimilarSessionsModal**

```typescript
// src/components/sessions/SimilarSessionsModal.tsx

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { SimilarSession } from '@/types';
import { useTranslation } from 'react-i18next';
import { SessionItemCompact } from './SessionItemCompact';

interface SimilarSessionsModalProps {
  sessionId: string;
  onClose: () => void;
}

export function SimilarSessionsModal({ sessionId, onClose }: SimilarSessionsModalProps) {
  const { t } = useTranslation();
  const [similar, setSimilar] = useState<SimilarSession[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<SimilarSession[]>('find_similar_sessions', {
      sessionId,
      limit: 5,
    })
      .then(setSimilar)
      .catch(() => setSimilar([]))
      .finally(() => setLoading(false));
  }, [sessionId]);

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-panel" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{t('insights.similarSessions')}</h3>
          <button className="modal-close" onClick={onClose}>×</button>
        </div>

        <div className="modal-body">
          {loading ? (
            <div className="modal-loading">{t('common.loading')}</div>
          ) : similar.length === 0 ? (
            <div className="modal-empty">{t('insights.noSimilarSessions')}</div>
          ) : (
            <div className="similar-sessions-list">
              {similar.map((item) => (
                <div key={item.session_id} className="similar-session-item">
                  <div className="similarity-header">
                    <span className="similarity-score">
                      {Math.round(item.similarity_score * 100)}% {t('insights.match')}
                    </span>
                    <span className="similarity-reason">
                      {item.similarity_reason}
                    </span>
                  </div>
                  <SessionItemCompact session={item.session} />
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button className="btn-primary" onClick={onClose}>
            {t('common.close')}
          </button>
        </div>
      </div>
    </div>
  );
}

// Compact session item for similar sessions list
function SessionItemCompact({ session }: { session: SimilarSession['session'] }) {
  return (
    <div className="session-item-compact">
      <div className="session-compact-header">
        <span className="session-topic">
          {session.stats?.topic || t('insights.untitled')}
        </span>
        <span className="session-date">
          {new Date(session.created_at).toLocaleDateString()}
        </span>
      </div>
      {session.stats && (
        <div className="session-compact-stats">
          <span>{session.stats.duration_minutes}m</span>
          <span>{session.stats.message_count} msgs</span>
        </div>
      )}
    </div>
  );
}
```

**Step 2: Add "Find Similar" button to SessionItem**

```typescript
// In src/components/sessions/SessionItem.tsx

import { useState } from 'react';
import { SimilarSessionsModal } from './SimilarSessionsModal';

export function SessionItem({ session }: { session: SessionMeta }) {
  const [showSimilar, setShowSimilar] = useState(false);

  // In the action buttons section
  return (
    <div className="session-item">
      {/* existing content */}

      <div className="session-actions">
        <button
          className="action-btn"
          onClick={() => setShowSimilar(true)}
          title={t('insights.findSimilar')}
        >
          🔍 {t('insights.findSimilar')}
        </button>
        {/* other action buttons */}
      </div>

      {showSimilar && (
        <SimilarSessionsModal
          sessionId={session.id}
          onClose={() => setShowSimilar(false)}
        />
      )}
    </div>
  );
}
```

**Step 3: Commit**

```bash
git add src/components/sessions/SimilarSessionsModal.tsx src/components/sessions/SessionItem.tsx
git commit -m "feat(frontend): add SimilarSessionsModal and Find Similar button"
```

---

## Frontend: Translations

### Task 15: Add i18n translations

**Files:**
- Modify: `src/i18n/locales/en.json`
- Modify: `src/i18n/locales/zh.json`

**Step 1: Add English translations**

```json
// src/i18n/locales/en.json

{
  "insights": {
    "title": "Session Insights",
    "tabs": {
      "overview": "Overview",
      "projects": "Projects",
      "workflow": "Workflow",
      "content": "Content"
    },
    "allProjects": "All Projects",
    "noData": "No sessions available for analysis. Start some coding sessions first!",
    "loadError": "Failed to load insights",
    "loading": "Analyzing sessions...",
    "emptyTitle": "No Session Data Yet",
    "emptyMessage": "Start using Claude Code in your projects. After a few sessions, we'll show you insights about your work patterns.",
    "errorTitle": "Couldn't Load Insights",
    "sessions": "Sessions",
    "totalTime": "Total Time",
    "avgDuration": "Avg Duration",
    "topCommands": "Top Commands",
    "topFiles": "Most Edited Files",
    "toolUsage": "Tool Usage",
    "commonSequences": "Common Command Sequences",
    "toolDistribution": "Tool Distribution",
    "sessionLength": "Session Length Distribution",
    "quickSessions": "Quick (<10m)",
    "mediumSessions": "Medium (10-60m)",
    "deepSessions": "Deep (>60m)",
    "taskTypes": "Task Types",
    "keyConcepts": "Key Concepts",
    "promptPatterns": "Repeated Prompt Patterns",
    "sessionsAnalyzed": "{{count}} sessions analyzed",
    "overviewTitle": "Overview",
    "totalProjects": "Total Projects",
    "totalSessions": "Total Sessions",
    "topTaskType": "Top Task Type",
    "topConcept": "Top Concept",
    "mostActiveProject": "Most Active Project",
    "topCommandSequence": "Most Common Command Sequence",
    "similarSessions": "Similar Sessions",
    "match": "match",
    "noSimilarSessions": "No similar sessions found",
    "findSimilar": "Find Similar",
    "untitled": "Untitled"
  }
}
```

**Step 2: Add Chinese translations**

```json
// src/i18n/locales/zh.json

{
  "insights": {
    "title": "会话洞察",
    "tabs": {
      "overview": "概览",
      "projects": "项目",
      "workflow": "工作流",
      "content": "内容"
    },
    "allProjects": "所有项目",
    "noData": "没有可用于分析的会话。先开始一些编程会话吧！",
    "loadError": "加载洞察失败",
    "loading": "正在分析会话...",
    "emptyTitle": "还没有会话数据",
    "emptyMessage": "开始在项目中使用 Claude Code。几个会话后，我们会显示您的工作模式洞察。",
    "errorTitle": "无法加载洞察",
    "sessions": "会话",
    "totalTime": "总时长",
    "avgDuration": "平均时长",
    "topCommands": "常用命令",
    "topFiles": "最常编辑的文件",
    "toolUsage": "工具使用",
    "commonSequences": "常见命令序列",
    "toolDistribution": "工具分布",
    "sessionLength": "会话时长分布",
    "quickSessions": "快速 (<10分钟)",
    "mediumSessions": "中等 (10-60分钟)",
    "deepSessions": "深度 (>60分钟)",
    "taskTypes": "任务类型",
    "keyConcepts": "关键概念",
    "promptPatterns": "重复的提示词模式",
    "sessionsAnalyzed": "已分析 {{count}} 个会话",
    "overviewTitle": "概览",
    "totalProjects": "总项目数",
    "totalSessions": "总会话数",
    "topTaskType": "主要任务类型",
    "topConcept": "主要概念",
    "mostActiveProject": "最活跃项目",
    "topCommandSequence": "最常见命令序列",
    "similarSessions": "相似会话",
    "match": "匹配",
    "noSimilarSessions": "未找到相似会话",
    "findSimilar": "查找相似",
    "untitled": "未命名"
  }
}
```

**Step 3: Commit**

```bash
git add src/i18n/locales/
git commit -m "feat(i18n): add insights translations for English and Chinese"
```

---

## Frontend: Styles

### Task 16: Add insights styles

**Files:**
- Create: `src/components/sessions/insights/InsightsTab.css`

**Step 1: Create styles file**

```css
/* src/components/sessions/insights/InsightsTab.css */

/* Empty states */
.insights-empty,
.insights-error,
.insights-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  text-align: center;
}

.empty-icon,
.error-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

/* Header */
.insights-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

.insights-header h2 {
  margin: 0;
}

.insights-header select {
  padding: 0.5rem 1rem;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
}

/* Navigation */
.insights-nav {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 1.5rem;
  border-bottom: 1px solid var(--border-color);
}

.insights-nav button {
  padding: 0.75rem 1rem;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.insights-nav button:hover {
  color: var(--text-primary);
}

.insights-nav button.active {
  color: var(--accent-color);
  border-bottom-color: var(--accent-color);
}

/* Project cards */
.project-stats-panel {
  display: grid;
  gap: 1.5rem;
}

.project-card {
  background: var(--bg-card);
  border-radius: 8px;
  padding: 1.5rem;
  border: 1px solid var(--border-color);
}

.project-card h3 {
  margin: 0 0 1rem 0;
}

.stats-summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.stat-item {
  text-align: center;
}

.stat-value {
  display: block;
  font-size: 1.5rem;
  font-weight: 600;
}

.stat-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.stats-section {
  margin-top: 1.5rem;
}

.stats-section h4 {
  margin: 0 0 0.75rem 0;
  font-size: 1rem;
}

/* Bar charts */
.bar-chart {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.bar-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.bar-label {
  min-width: 120px;
  font-size: 0.875rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bar-container {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.bar-track {
  flex: 1;
  height: 24px;
  background: var(--bg-tertiary);
  border-radius: 4px;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  background: var(--accent-color);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.bar-fill--files {
  background: var(--color-blue);
}

.bar-value {
  min-width: 30px;
  text-align: right;
  font-size: 0.875rem;
  font-weight: 500;
}

/* Tool usage chart */
.tool-usage-chart,
.tool-distribution {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.tool-item,
.tool-dist-item {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.tool-name,
.tool-dist-name {
  min-width: 80px;
  font-size: 0.875rem;
}

.tool-bar {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.tool-bar-fill {
  height: 20px;
  background: var(--color-green);
  border-radius: 4px;
  transition: width 0.3s ease;
}

.tool-percent,
.tool-dist-percent {
  min-width: 40px;
  text-align: right;
  font-size: 0.875rem;
}

/* Sequence flow */
.sequence-flow {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.sequence-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-radius: 6px;
}

.sequence-steps {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex: 1;
}

.step-badge {
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 0.875rem;
}

.arrow {
  color: var(--text-secondary);
}

.sequence-count {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

/* Length histogram */
.length-histogram {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.histogram-bar {
  display: grid;
  grid-template-columns: 100px 1fr 40px;
  gap: 0.75rem;
  align-items: center;
}

.bar-label {
  font-size: 0.875rem;
}

.bar-fill--quick {
  background: var(--color-green);
}

.bar-fill--medium {
  background: var(--color-yellow);
}

.bar-fill--deep {
  background: var(--color-blue);
}

/* Task badges */
.task-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.task-badge {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  border-radius: 20px;
  font-size: 0.875rem;
}

.task-count {
  font-weight: 600;
}

.task-percent {
  font-size: 0.75rem;
  opacity: 0.7;
}

/* Concept cloud */
.concept-cloud {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
  padding: 1rem;
  background: var(--bg-secondary);
  border-radius: 8px;
}

.concept-tag {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  transition: transform 0.2s;
}

.concept-tag:hover {
  transform: scale(1.05);
}

.concept-count {
  font-size: 0.75rem;
  opacity: 0.6;
}

/* Pattern list */
.pattern-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.pattern-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border-radius: 4px;
}

.pattern-text {
  font-size: 0.875rem;
}

.pattern-count {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

/* Overview cards */
.overview-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 2rem;
}

.overview-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem;
  background: var(--bg-card);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.card-icon {
  font-size: 2rem;
}

.card-value {
  font-size: 1.5rem;
  font-weight: 600;
}

.card-label {
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.overview-section {
  margin-top: 2rem;
}

.overview-section h3 {
  margin-bottom: 1rem;
}

.project-highlight,
.sequence-highlight {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: var(--bg-secondary);
  border-radius: 6px;
}

.project-name {
  font-weight: 600;
}

.project-stats {
  color: var(--text-secondary);
  font-size: 0.875rem;
}

.sequence-count {
  padding: 0.25rem 0.5rem;
  background: var(--bg-tertiary);
  border-radius: 4px;
  font-size: 0.875rem;
}

/* Similar sessions modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-panel {
  background: var(--bg-primary);
  border-radius: 12px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
}

.modal-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
}

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding: 1.5rem;
}

.modal-footer {
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-color);
}

.similar-sessions-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.similar-session-item {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}

.similarity-header {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  font-size: 0.875rem;
}

.similarity-score {
  font-weight: 600;
  color: var(--accent-color);
}

.similarity-reason {
  color: var(--text-secondary);
}

.session-item-compact {
  padding: 0.75rem;
}

.session-compact-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}

.session-topic {
  font-weight: 500;
}

.session-date {
  font-size: 0.75rem;
  color: var(--text-secondary);
}

.session-compact-stats {
  display: flex;
  gap: 1rem;
  font-size: 0.75rem;
  color: var(--text-secondary);
}

/* Find Similar button */
.session-actions .action-btn {
  padding: 0.375rem 0.75rem;
  font-size: 0.875rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
}

.session-actions .action-btn:hover {
  background: var(--bg-tertiary);
}
```

**Step 2: Import styles in InsightsTab**

```typescript
// Add at top of src/components/sessions/insights/InsightsTab.tsx

import './InsightsTab.css';
```

**Step 3: Commit**

```bash
git add src/components/sessions/insights/InsightsTab.css
git commit -m "feat(styles): add comprehensive insights styles"
```

---

## Final Steps

### Task 17: Final testing and polish

**Step 1: Run TypeScript compilation**

```bash
cd src && npm run tsc
```

Expected: No errors

**Step 2: Run Rust tests**

```bash
cd src-tauri && cargo test
```

Expected: All tests pass

**Step 3: Build application**

```bash
npm run tauri build
```

Expected: Build succeeds

**Step 4: Manual testing checklist**

- [ ] Open Session Manager page
- [ ] Click on "Insights" tab
- [ ] Verify overview panel shows summary cards
- [ ] Navigate to Projects tab - verify project stats
- [ ] Navigate to Workflow tab - verify patterns
- [ ] Navigate to Content tab - verify analysis
- [ ] Click "Find Similar" on a session
- [ ] Verify similar sessions modal shows matches
- [ ] Test with no sessions - verify empty state
- [ ] Test project filtering

**Step 5: Final commit**

```bash
git add .
git commit -m "feat(insights): complete Phase 3 Session Insights implementation

Implement comprehensive session analysis system:
- Project statistics (commands, files, tools)
- Workflow pattern detection (sequences, tool flows)
- Content analysis (task types, concepts, prompt patterns)
- Similar session finder with multi-factor scoring
- Full UI with Overview/Projects/Workflow/Content tabs
- i18n support for English and Chinese
- Comprehensive error handling and loading states

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Summary

This implementation plan builds the **Session Insights & Pattern Detection** feature through:

**Backend (6 tasks):**
1. Analysis module structure
2. ProjectAnalyzer - aggregates by project
3. WorkflowAnalyzer - detects sequences and patterns
4. ContentAnalyzer - classifies tasks and extracts concepts
5. SimilarityFinder - multi-factor session matching
6. Tauri commands for all analysis types

**Frontend (9 tasks):**
7. TypeScript types for all data structures
8. InsightsTab main component with navigation
9. Integration with SessionManagerPage
10. ProjectStatsPanel with bar charts
11. WorkflowPatternsPanel with sequence visualization
12. ContentAnalysisPanel with badges and cloud
13. OverviewPanel with summary cards
14. SimilarSessionsModal with similarity scoring
15. i18n translations (en + zh)
16. Comprehensive CSS styles

**Testing & Polish (1 task):**
17. Final testing, compilation, and commit

**Total: 17 tasks, ~35-40 implementation steps**

All following TDD, with bite-sized commits, DRY principles, and YAGNI mindset.
