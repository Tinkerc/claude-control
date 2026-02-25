# Session Insights & Pattern Detection - Design Document

**Date:** 2026-02-25
**Status:** Design Approved
**Phase:** 3 - Insights & Pattern Detection

---

## Overview

Build an intelligent session analysis system that learns from Claude Code session history and provides actionable insights about work patterns, project-specific behaviors, and content trends.

### Problem Statement

Users want to:
- Understand their work patterns across projects
- Learn what commands, tools, and files they use most frequently
- Identify repeated workflows that could be automated
- Find similar sessions to reference previous work
- Extract project knowledge automatically

### Solution

A rule-based analysis engine (with optional LLM upgrade path) that:
1. Aggregates session data by project
2. Detects workflow patterns (command sequences, tool flows)
3. Analyzes content (task types, concepts, prompt patterns)
4. Finds similar sessions based on multiple factors

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (React)                        │
├─────────────────────────────────────────────────────────────┤
│  SessionManagerPage                                          │
│  ├── Tab 1: Sessions (existing)                             │
│  └── Tab 2: Insights (NEW)                                  │
│      ├── Overview - Cross-pattern summary                   │
│      ├── Projects - Project-specific stats                  │
│      ├── Workflow - Command sequences & patterns            │
│      └── Content - Task types, concepts, patterns           │
└────────────────────┬────────────────────────────────────────┘
                     │ Tauri Commands
┌────────────────────▼────────────────────────────────────────┐
│                   Backend (Rust)                            │
├─────────────────────────────────────────────────────────────┤
│  Commands: get_project_stats, get_workflow_patterns,        │
│           get_content_analysis, find_similar_sessions       │
├─────────────────────────────────────────────────────────────┤
│  Analysis Services                                          │
│  ├── ProjectAnalyzer    - Aggregates by project            │
│  ├── WorkflowAnalyzer   - Detects sequences & patterns     │
│  ├── ContentAnalyzer    - Classifies, extracts concepts    │
│  └── SimilarityFinder   - Multi-factor similarity          │
├─────────────────────────────────────────────────────────────┤
│  SessionService (existing) - Raw session data               │
└─────────────────────────────────────────────────────────────┘
```

---

## Data Structures

### Core Data Types

```rust
// Project statistics
pub struct ProjectStats {
    pub project_path: String,
    pub session_count: usize,
    pub total_duration_minutes: i64,
    pub top_commands: Vec<CommandFrequency>,
    pub top_files: Vec<FileFrequency>,
    pub top_tools: Vec<ToolFrequency>,
    pub avg_session_duration: f64,
}

// Workflow patterns
pub struct WorkflowPatterns {
    pub command_sequences: Vec<CommandSequence>,
    pub tool_usage_distribution: Vec<ToolFrequency>,
    pub session_length_distribution: LengthDistribution,
    pub most_common_sequences: Vec<SequencePattern>,
}

// Content analysis
pub struct ContentAnalysis {
    pub task_classification: Vec<TaskCategory>,      // bug_fix, feature, refactor...
    pub concepts: Vec<ConceptEntry>,                 // "authentication": 12 mentions
    pub prompt_patterns: Vec<PromptPattern>,         // "add tests for": 8 occurrences
    pub total_sessions_analyzed: usize,
}

// Similar sessions
pub struct SimilarSession {
    pub session_id: String,
    pub similarity_score: f64,        // 0.0 - 1.0
    pub similarity_reason: String,    // "same files edited", "similar commands"
    pub session: SessionMeta,
}
```

---

## Analysis Algorithms (Rule-Based)

### ProjectAnalyzer

Groups sessions by project path and aggregates:
- Command frequency (count + percentage)
- File edit frequency
- Tool usage distribution
- Time spent per project
- Average session duration

### WorkflowAnalyzer

Detects patterns in how users work:
- **Command sequences**: Finds 2-3 command sequences that repeat (e.g., "git add → git commit → git push")
- **Tool flows**: Detects common tool usage patterns (e.g., "Write → Bash → pytest")
- **Session lengths**: Categorizes sessions (quick <10min, medium 10-60min, deep >60min)

### ContentAnalyzer

Analyzes user prompt content:
- **Task classification**: Keyword-based classification (bug_fix, feature, refactor, documentation, testing, review, other)
- **Concept extraction**: Identifies technical terms (api, auth, database, endpoint, etc.)
- **Prompt patterns**: Detects repeated phrasing patterns (e.g., "add tests for X", "implement Y")

### SimilarityFinder

Finds similar sessions using multi-factor scoring:
- Same project (+0.3)
- Shared file edits (+0.0 to +0.4 based on overlap)
- Similar task type (+0.2)
- Shared commands (+0.0 to +0.1 based on overlap)

---

## Frontend Components

### Main Structure

```
InsightsTab
├── InsightsHeader (project selector)
├── InsightsNav (view mode tabs)
└── Content Panels
    ├── OverviewPanel (summary dashboard)
    ├── ProjectStatsPanel (detailed project stats)
    ├── WorkflowPatternsPanel (sequences and flows)
    └── ContentAnalysisPanel (tasks, concepts, patterns)
```

### Visual Components

| Component | Purpose |
|-----------|---------|
| CommandStatsChart | Horizontal bar chart of top commands |
| FileStatsChart | Horizontal bar chart of most edited files |
| ToolUsagePieChart | Pie/donut chart of tool distribution |
| SequenceFlowView | Visual flow diagram of command sequences |
| SessionLengthHistogram | Bar histogram of session durations |
| TaskTypeBadges | Color-coded badges for task categories |
| ConceptCloud | Word cloud of key concepts |
| PromptPatternList | List of repeated prompt patterns |
| SimilarSessionsModal | Modal showing similar sessions |

---

## Error Handling

### Edge Cases

| Case | Handling |
|------|----------|
| No sessions | Empty state with helpful message |
| Single session | Limited insights (no patterns) |
| Corrupted data | Skip bad sessions, warn if >50% corrupted |
| Empty fields | Show "N/A" or hide section |
| Long paths | Truncate with ellipsis |
| Too many results | Limit to top 10 |
| Slow analysis | Show loading, timeout after 10s |

### Validation

- Validate session data integrity before analysis
- Filter out sessions with obviously corrupted stats
- Return partial results if some analyses fail
- Provide user-friendly error messages

---

## Testing Strategy

### Backend Unit Tests

- Test grouping logic (by project, by pattern)
- Test aggregation (counts, percentages)
- Test sequence detection
- Test similarity scoring
- Test edge cases (empty, single, corrupted)

### Frontend Tests

- Test loading/error/empty states
- Test data rendering
- Test user interactions (filter, drill down)
- Test "Find Similar" flow

### Integration Tests

- Full analysis flow with real session data
- Cross-tab navigation
- Project filtering
- Similar session modal

---

## Future Enhancements

### LLM Integration (Optional)

Add opt-in LLM analysis for:
- Better task classification (understand context, not just keywords)
- Semantic similarity (embeddings for finding similar sessions)
- Smarter concept extraction (identify domain terms automatically)
- Pattern explanation (explain WHY a pattern is interesting)

### Additional Features

- Time-based trends (are you using certain tools more over time?)
- Project comparison (compare patterns across projects)
- Skill suggestion (suggest skills/automations based on patterns)
- CLAUDE.md generation (auto-generate project documentation)

---

## Implementation Timeline

### Week 1-2: Core Insights (Phase 3A)

**Backend**
- Create analysis module
- Implement all analyzers
- Add Tauri commands
- Write unit tests

**Frontend**
- Create insights components
- Build all panels
- Add tab navigation
- Add i18n translations

### Week 2-3: Polish & Similar Sessions (Phase 3B)

- Add SimilarSessionsModal
- Add "Find Similar" button
- Add caching for performance
- Visual polish and animations
- Error handling polish

### Future: LLM Integration (Phase 3C - Optional)

- API key configuration
- LLM analyzer service
- Opt-in toggle
- Fallback to rule-based

---

## Success Criteria

- [ ] User can view insights by project
- [ ] User can see common command sequences
- [ ] User can see task type breakdown
- [ ] User can find similar sessions
- [ ] Analysis completes in <3 seconds for 1000 sessions
- [ ] All UI is responsive with proper loading states
- [ ] Error cases handled gracefully

---

## Design Decisions

### Why rule-based first?

1. **Privacy**: No data leaves the machine
2. **Speed**: No API calls, instant results
3. **Cost**: No API usage fees
4. **Reliability**: Works offline
5. **Upgrade path**: Can add LLM later if needed

### Why no new database tables?

1. All needed data already exists in `sessions` table
2. Simpler migration path
3. Can add caching later if performance is an issue
4. YAGNI principle

### Why separate tabs?

1. Cleaner mental model for users
2. Each tab focuses on one aspect
3. Can show cross-patterns in Overview tab
4. Easy to add new insight types later
