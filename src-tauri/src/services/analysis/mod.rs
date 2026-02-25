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
