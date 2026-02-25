# Session Management Feature Plan (Simplified)

**Created:** 2026-02-24
**Updated:** 2026-02-25
**Status:** Phase 2 Completed ✅

---

## 🎯 Goal

Build a simple, focused session management feature for **Claude Code only**. This plan prioritizes essential features that provide immediate value without over-engineering.

---

## 📊 Current State

### ✅ Completed (Phase 1 + Phase 2)
- Date Grouping Timeline (Today, Yesterday, This Week, Older)
- Session Statistics Badge (duration, messages, files, commands)
- Claude Code provider parser with metadata extraction
- **Enhanced Search (files, commands, tools, keyword, project directory)**

### ⏳ Pending
- Export to Markdown
- Quick Resume

---

## 🎯 Feature Roadmap

### Phase 1: Core Session Browser ✅ DONE

| Feature | Description | Status |
|---------|-------------|--------|
| Date Grouping | Group by Today, Yesterday, This Week, Older | ✅ |
| Stats Badge | Show duration, messages, files, commands | ✅ |
| Claude Parser | Extract metadata from Claude Code sessions | ✅ |

### Phase 2: Enhanced Search & Filtering ✅ DONE (2026-02-25)

| Feature | Description | Priority | Status |
|---------|-------------|----------|--------|
| Search by Files | Find sessions that modified specific files | 🔴 High | ✅ |
| Search by Commands | Find sessions using specific commands | 🟡 Medium | ✅ |
| Filter by Tools | Filter by tools used (Read, Write, Bash, etc.) | 🟢 Low | ✅ |
| **Keyword Search** | Search title, summary, session ID | 🔴 High | ✅ |
| **Project Directory** | Filter by project directory path | 🟡 Medium | ✅ |

### Phase 3: Quick Actions (Next)

| Feature | Description | Priority |
|---------|-------------|----------|
| Export Session | Export to Markdown | 🟡 Medium |
| Quick Resume | Copy working directory + context | 🔴 High |

---

## 🏗️ Technical Implementation

### Provider Support: Claude Code Only

```
src-tauri/src/session_manager/providers/
├── claude.rs       ✅ Implemented (primary provider)
├── codex.rs        ❌ Not needed
├── opencode.rs     ❌ Not needed
├── openclaw.rs     ❌ Not needed
└── gemini.rs       ❌ Not needed
```

### Database Schema

```sql
-- Core session table (Claude Code only)
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_path TEXT,
    created_at TEXT,
    last_active_at TEXT,

    -- Statistics
    duration_minutes INTEGER,
    message_count INTEGER,
    user_message_count INTEGER,
    assistant_message_count INTEGER,

    -- Activity data (JSON)
    tools_used TEXT,        -- ["Write", "Read", "Bash"]
    files_modified TEXT,    -- ["src/main.rs", "src/lib.rs"]
    commands_executed TEXT, -- ["/clear", "commit"]

    -- Meta
    model TEXT,             -- "glm-4.7"
    topic TEXT,             -- from first user message
    token_usage TEXT        -- {"input": 123, "output": 456}
);
```

### Tauri Commands

```rust
// src-tauri/src/commands/session_manager.rs

// ✅ Existing
pub async fn list_sessions() -> Result<Vec<SessionMeta>, String>;
pub async fn get_session_messages(
    providerId: String,
    sourcePath: String,
) -> Result<Vec<SessionMessage>, String>;
pub async fn launch_session_terminal(
    command: String,
    cwd: Option<String>,
    custom_config: Option<String>,
) -> Result<bool, String>;

// ✅ Phase 2 - Search (NEW)
pub async fn search_sessions(
    query: SessionSearchQuery,
) -> Result<SessionSearchResult, String>;

// ⏳ Phase 3 - Actions
pub async fn export_session(id: String) -> Result<String, String>;
pub async fn get_resume_context(id: String) -> Result<ResumeContext, String>;
```

### Frontend Components

```
src/components/sessions/
├── SessionManagerPage.tsx    ✅ Main page
├── SessionItem.tsx           ✅ Session list item
├── SessionStatsBadge.tsx     ✅ Statistics display
├── SessionMessageItem.tsx    ✅ Message detail view
├── SessionToc.tsx            ✅ Table of contents
├── SessionSearchBar.tsx      ✅ Phase 2 - Enhanced search
├── utils.ts                  ✅ Utility functions
└── ActionButtons.tsx         ⏳ Phase 3 - Export/Resume
```

---

## 📅 Implementation Timeline

### ✅ Week 1: Phase 1 Complete
- Date grouping UI
- Statistics badge
- Claude Code parser

### ✅ Week 2: Phase 2 Complete (2026-02-25)
- Backend search API (`search_sessions` command)
- Frontend SearchBar component with 5 filters
- Keyword search (title, summary, session ID)
- Project directory filter
- Files/commands/tools filters
- i18n translations (EN/ZH/JA)

### ⏳ Week 3+: Phase 3 - Actions
- Export to Markdown
- Quick Resume context

---

## ✅ Phase 2 Summary

**Backend (Rust):**
- `SessionSearchQuery` struct with keyword, files, commands, tools, project, provider, time range
- `SessionSearchResult` struct with matched sessions and metadata
- `search_sessions()` function with comprehensive filtering logic
- `matches_query()` helper for multi-criteria matching
- Tauri command registered in `lib.rs`

**Frontend (TypeScript/React):**
- `SessionSearchBar.tsx` - Advanced search with 5 filter types
- `useBackendSessionSearch.ts` - Hook for backend search integration
- Updated `SessionManagerPage.tsx` - Integrated both frontend and backend search
- i18n translations for EN, ZH, JA

**Files Created/Modified:**
- `src-tauri/src/session_manager/mod.rs` - Search types and functions
- `src-tauri/src/commands/session_manager.rs` - Tauri command
- `src/components/sessions/SessionSearchBar.tsx` - New component
- `src/hooks/useBackendSessionSearch.ts` - New hook
- `src/types.ts`, `src/lib/api/sessions.ts` - Type definitions
- `src/i18n/locales/{en,zh,ja}.json` - Translations

---

## 📝 Design Principles

1. **Claude Code Only** - Single provider, focused implementation
2. **Simple First** - Core features before advanced ones
3. **Local Only** - No cloud sync, privacy-focused
4. **Progressive** - Build on what exists, don't rewrite
5. **Multi-language** - Full i18n support (EN/ZH/JA)
