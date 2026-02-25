// Placeholder for workflow_analyzer - will be implemented in Task 3

use serde::{Serialize, Deserialize};

/// Workflow pattern detection analyzer
pub struct WorkflowAnalyzer;

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

// Re-use ToolFrequency from project_analyzer
use super::project_analyzer::ToolFrequency;
