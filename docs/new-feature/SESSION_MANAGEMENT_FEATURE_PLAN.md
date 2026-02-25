# Session Management Feature Plan

**Created:** 2026-02-24  
**Status:** Phase 1 MVP Completed ✅

---

## 📊 Current State Analysis

### Existing Foundation
- ✅ Existing `SessionManagerPage` component with session list/detail view
- ✅ Backend support for 5 providers (Claude, Codex, OpenCode, OpenClaw, Gemini)
- ✅ Session scanning from `~/.claude/projects/**/*.jsonl`
- ✅ Message loading and terminal launch capabilities
- ✅ Search, filter, and provider selection UI

### Extracted Data Insights

From sample data (`29 session_tmps`, `240 project_sessions`, `580 history records`):

```json
{
  "session_id": "uuid",
  "project_path": "projects/-Users-...",
  "duration_minutes": 477,
  "message_count": 221,
  "user_message_count": 81,
  "assistant_message_count": 140,
  "tools_used": ["Write", "Read", "TaskCreate", "Bash", "Glob"],
  "files_modified": ["/agent_watcher.rs", "/mod.rs"],
  "commands_executed": ["/clear", "callback"],
  "token_usage": {"input_tokens": 85609, "output_tokens": 8119},
  "model": "glm-4.7",
  "first_prompt": "..."
}
```

---

## 🎯 Feature List

### **Phase 1: Enhanced Session Browser** (Immediate Value)

| ID | Feature | Description | Priority |
|----|---------|-------------|----------|
| 1.1 | **Date Grouping Timeline** | Group sessions by date (Today, Yesterday, Feb 22, Feb 18...) | 🔴 High |
| 1.2 | **Project/Topic Grouping** | Group sessions by `topic` field (e.g., all `badminton-video-cut` sessions) | 🔴 High |
| 1.3 | **Session Statistics Badge** | Show message count, duration, files modified on each session item | 🔴 High |
| 1.4 | **Enhanced Search** | Search by: topic, files modified, commands executed, tools used | 🔴 High |
| 1.5 | **Session TOC** | Extract user messages as navigation points for long sessions | 🟡 Medium |

### **Phase 2: Analytics & Insights** (Differentiation)

| ID | Feature | Description | Priority |
|----|---------|-------------|----------|
| 2.1 | **Usage Dashboard** | Statistics: total sessions, messages, unique projects, time spent | 🔴 High |
| 2.2 | **Top Projects Ranking** | Rank projects by session count, total duration, activity level | 🟡 Medium |
| 2.3 | **Work Patterns Chart** | Peak hours, session duration trends, daily/weekly patterns | 🟡 Medium |
| 2.4 | **File Activity Heatmap** | Most modified files/dirs across sessions | 🟡 Medium |
| 2.5 | **Tool Usage Analytics** | Most used tools (Write, Read, Bash, TaskCreate, etc.) | 🟢 Low |

### **Phase 3: Memory & Knowledge** (Core Intelligence)

| ID | Feature | Description | Priority |
|----|---------|-------------|----------|
| 3.1 | **Auto-Generated Project Memory** | Extract common commands, tools used per project | 🔴 High |
| 3.2 | **Prompt Pattern Detection** | Identify repeated task patterns (e.g., `/phase-end` workflow) | 🟡 Medium |
| 3.3 | **Session Tags** | Auto-tag sessions by activity type: coding, debugging, planning | 🟡 Medium |
| 3.4 | **Token Usage Tracking** | Track token consumption per session/project | 🟢 Low |

### **Phase 4: Session Actions** (Productivity)

| ID | Feature | Description | Priority |
|----|---------|-------------|----------|
| 4.1 | **Export Session** | Export to Markdown/JSON for documentation | 🟡 Medium |
| 4.2 | **Session Comparison** | Compare two sessions (files changed, time spent) | 🟢 Low |
| 4.3 | **Quick Resume** | Copy working directory + last task context | 🔴 High |
| 4.4 | **Session Archive** | Mark sessions as archived, cleanup old tmp files | 🟢 Low |

### **Phase 5: Integration** (Ecosystem)

| ID | Feature | Description | Priority |
|----|---------|-------------|----------|
| 5.1 | **CLAUDE.md Generator** | Auto-generate project CLAUDE.md from session history | 🟡 Medium |
| 5.2 | **Skill Recommendation** | Suggest skills based on repeated tasks | 🟢 Low |
| 5.3 | **Error Pattern Detection** | Track recurring errors/failures across sessions | 🟡 Medium |

---

## 🏗️ Technical Implementation

### Backend (Rust/Tauri)

#### Database Schema Extension

```sql
-- Add new columns to existing sessions table
ALTER TABLE sessions ADD COLUMN duration_minutes INTEGER;
ALTER TABLE sessions ADD COLUMN message_count INTEGER;
ALTER TABLE sessions ADD COLUMN user_message_count INTEGER;
ALTER TABLE sessions ADD COLUMN assistant_message_count INTEGER;
ALTER TABLE sessions ADD COLUMN tools_used TEXT; -- JSON array
ALTER TABLE sessions ADD COLUMN files_modified TEXT; -- JSON array
ALTER TABLE sessions ADD COLUMN commands_executed TEXT; -- JSON array
ALTER TABLE sessions ADD COLUMN token_usage TEXT; -- JSON object
ALTER TABLE sessions ADD COLUMN model TEXT;
ALTER TABLE sessions ADD COLUMN topic TEXT;
```

#### New Tauri Commands

```rust
// src-tauri/src/commands/session_manager.rs

#[tauri::command]
pub async fn get_session_analytics() -> Result<SessionAnalytics, String>;

#[tauri::command]
pub async fn get_project_rankings() -> Result<Vec<ProjectRanking>, String>;

#[tauri::command]
pub async fn export_session(sessionId: String, format: String) -> Result<String, String>;

#[tauri::command]
pub async fn get_work_patterns() -> Result<WorkPatterns, String>;
```

### Frontend (React/TypeScript)

#### New Components

```
src/components/sessions/
├── SessionTimeline.tsx        # Date-grouped list view
├── SessionAnalytics.tsx       # Dashboard with charts
├── ProjectMemoryCard.tsx      # Auto-generated memory display
├── SessionExportDialog.tsx    # Export functionality
├── SessionStatsBadge.tsx      # Statistics badge component
└── SessionTocDialog.tsx       # Table of contents for sessions
```

#### New Hooks

```typescript
src/hooks/
├── useSessionAnalytics.ts     // Query analytics data
├── useProjectMemory.ts        // Auto-generated memory
├── useSessionExport.ts        // Export functionality
└── useWorkPatterns.ts         // Work patterns analysis
```

---

## 📅 Implementation Timeline

### Week 1-2: Phase 1 (MVP) ✅ COMPLETED
- ✅ 1.1 Date Grouping Timeline
- ✅ 1.3 Session Statistics Badge
- ⏳ 1.4 Enhanced Search (basic search exists, file/command/tool search pending)
- ⏳ 1.2 Project/Topic Grouping (topic field added, grouping pending)
- ✅ 1.5 Session TOC (already existed)

### Week 3-4: Phase 2 (Analytics)
- ⏳ 2.1 Usage Dashboard
- ⏳ 2.2 Top Projects Ranking
- ⏳ 2.3 Work Patterns Chart

### Week 5-6: Phase 3 (Memory)
- ⏳ 3.1 Auto-Generated Project Memory
- ⏳ 3.2 Prompt Pattern Detection

### Week 7+: Phase 4-5 (Advanced)
- ⏳ 4.1 Export Session
- ⏳ 4.3 Quick Resume
- ⏳ 5.1 CLAUDE.md Generator

---

## ✅ Phase 1 Implementation Checklist

- [x] Create `SessionStatsBadge.tsx` component for statistics display
- [x] Update `SessionItem.tsx` to show statistics
- [x] Add `groupSessionsByDate` utility function
- [x] Update `SessionManagerPage.tsx` with date grouping UI
- [x] Update backend `SessionMeta` struct with new fields (duration, tools, etc.)
- [x] Update Claude provider parser to extract metadata
- [x] Update frontend `SessionMeta` type definition
- [x] Add translation keys for date grouping (today, yesterday)
- [x] TypeScript compilation passes

---

## 📝 Notes

- All features are **local-first** (no cloud sync by default)
- Privacy: Session content never leaves the local machine
- Performance: Use virtual scrolling for long session lists
- Accessibility: Ensure all new components are keyboard navigable
