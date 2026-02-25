use std::collections::HashSet;
use crate::session_manager::SessionMeta;
use serde::{Serialize, Deserialize};

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
            if session.session_id == target.session_id {
                continue;
            }

            let (score, reason) = Self::calculate_similarity(target, session);
            if score > 0.3 {
                scored.push((
                    SimilarSession {
                        session_id: session.session_id.clone(),
                        similarity_score: score,
                        similarity_reason: reason,
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
        if a.project_dir == b.project_dir {
            score += 0.3;
            reasons.push("same project".to_string());
        }

        // Overlapping files?
        if let (Some(a_stats), Some(b_stats)) = (&a.stats, &b.stats) {
            // Use empty vec as default to avoid temporary value issues
            let empty = vec![];
            let a_files = a_stats.files_modified.as_ref().unwrap_or(&empty);
            let b_files = b_stats.files_modified.as_ref().unwrap_or(&empty);

            let overlap: HashSet<_> = a_files
                .iter()
                .filter(|f| b_files.contains(f))
                .collect();

            if !overlap.is_empty() {
                let overlap_ratio = overlap.len() as f64 /
                    a_files.len().max(1) as f64;
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
            let a_cmds = a_stats.commands_executed.as_ref().unwrap_or(&empty);
            let b_cmds = b_stats.commands_executed.as_ref().unwrap_or(&empty);

            let cmd_overlap: HashSet<_> = a_cmds
                .iter()
                .filter(|c| b_cmds.contains(c))
                .collect();

            if !cmd_overlap.is_empty() {
                let cmd_ratio = cmd_overlap.len() as f64 /
                    a_cmds.len().max(1) as f64;
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

/// Similar session result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimilarSession {
    pub session_id: String,
    pub similarity_score: f64,
    pub similarity_reason: String,
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
        topic: &str,
    ) -> SessionMeta {
        SessionMeta {
            provider_id: "claude".to_string(),
            session_id: format!("id-{}-{}", project, uuid::Uuid::new_v4()), // Use UUID for unique IDs
            project_dir: Some(project.to_string()),
            stats: Some(SessionStats {
                commands_executed: Some(commands.into_iter().map(String::from).collect()),
                files_modified: Some(files.into_iter().map(String::from).collect()),
                topic: Some(topic.to_string()),
                tools_used: None,
                duration_minutes: None,
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
    fn test_similarity_same_project() {
        let session_a = create_test_session("/proj/a", vec!["test"], vec!["a.rs"], "Fix bug");
        let session_b = create_test_session("/proj/a", vec!["test"], vec!["a.rs"], "Fix bug"); // Same project, same files, same topic
        let session_c = create_test_session("/proj/b", vec!["build"], vec!["c.rs"], "Add feature");

        let similar = SimilarityFinder::find_similar(&session_a, &[session_b.clone(), session_c.clone()], 10);

        // Debug output
        if similar.is_empty() {
            let (score_a_b, reason_a_b) = SimilarityFinder::calculate_similarity(&session_a, &session_b);
            let (score_a_c, reason_a_c) = SimilarityFinder::calculate_similarity(&session_a, &session_c);
            panic!("No similar sessions found. a->b: score={}, reason={}. a->c: score={}, reason={}",
                   score_a_b, reason_a_b, score_a_c, reason_a_c);
        }

        // session_b matches: same project (0.3) + shared files (0.4) + similar topic (0.2) = 0.9
        assert!(similar.len() >= 1);
        let match_b = similar.iter().find(|s| s.session_id == session_b.session_id).unwrap();
        assert!(match_b.similarity_score >= 0.8);
    }

    #[test]
    fn test_similarity_shared_files() {
        let session_a = create_test_session("/proj", vec!["test"], vec!["src/auth.rs", "src/user.rs"], "Fix auth");
        let session_b = create_test_session("/proj", vec!["build"], vec!["src/auth.rs"], "Fix auth");
        let session_c = create_test_session("/proj", vec!["test"], vec!["src/test.rs"], "Add test");

        let similar = SimilarityFinder::find_similar(&session_a, &[session_b.clone(), session_c.clone()], 10);

        // session_a and session_b share auth.rs (0.5 overlap ratio) and similar topic (0.2) + same project (0.3)
        let match_b = similar.iter().find(|s| s.session_id == session_b.session_id);
        assert!(match_b.is_some());
        assert!(match_b.unwrap().similarity_reason.contains("shared files"));
        assert!(match_b.unwrap().similarity_reason.contains("similar task type"));
    }

    #[test]
    fn test_empty_sessions() {
        let session_a = create_test_session("/proj", vec![], vec![], "test");
        let similar = SimilarityFinder::find_similar(&session_a, &[], 10);
        assert_eq!(similar.len(), 0);
    }
}
