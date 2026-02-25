// Placeholder for similarity_finder - will be implemented in Task 5

use serde::{Serialize, Deserialize};

/// Find similar sessions using multi-factor scoring
pub struct SimilarityFinder;

/// Similar session result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimilarSession {
    pub session_id: String,
    pub similarity_score: f64,
    pub similarity_reason: String,
}
