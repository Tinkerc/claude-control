# Phase 3: Quick Actions - Implementation Plan

**Created:** 2026-02-25
**Status:** Planning
**Provider:** Claude Code Only

---

## 🎯 Overview

Phase 3 adds productivity-focused actions to help users quickly export session data and resume previous work contexts.

---

## 📋 Feature List

### 3.1 Export Session (Priority: High)

**Description:** Export a session to Markdown format for documentation, sharing, or archival purposes.

**User Flow:**
1. User clicks "Export" button on a session item
2. System generates Markdown with:
   - Session metadata (date, duration, project, model)
   - User prompts as headings
   - Assistant responses as content
   - Files modified list
   - Commands executed
3. User can copy to clipboard or save to file

**UI Components:**
```
src/components/sessions/
├── SessionExportDialog.tsx     # Export preview and options
└── SessionItem.tsx             # Add export button
```

**Backend Command:**
```rust
#[tauri::command]
pub async fn export_session(
    session_id: String,
    format: ExportFormat,  // Markdown, JSON
    include_metadata: bool
) -> Result<String, String>
```

**Markdown Output Format:**
```markdown
# Claude Code Session

**Date:** 2025-02-25 14:30
**Duration:** 47 minutes
**Project:** ~/work/my-project
**Model:** glm-4.7

## Summary
- Messages: 81
- Files Modified: 5
- Commands Executed: 3

---

## Conversation

### User: Add authentication to the API

[... conversation content ...]

### Assistant Response

[... assistant content ...]

---

## Files Modified
- `src/auth/login.rs`
- `src/auth/middleware.rs`
- `src/api/routes.rs`

## Commands Executed
- `/clear`
- `cargo test`
- `cargo build`
```

---

### 3.2 Quick Resume (Priority: High)

**Description:** Quickly copy the working directory and last task context to resume a previous session.

**User Flow:**
1. User clicks "Resume" button on a session item
2. System displays resume dialog with:
   - Working directory path
   - Last user prompt/task
   - Files modified in that session
3. User can:
   - Copy directory path
   - Copy full context (directory + task description)
   - Open terminal in that directory

**UI Components:**
```
src/components/sessions/
├── SessionResumeDialog.tsx     # Resume context display
└── SessionItem.tsx             # Add resume button
```

**Backend Command:**
```rust
#[tauri::command]
pub async fn get_resume_context(
    session_id: String
) -> Result<ResumeContext, String>

#[derive(Serialize, Deserialize)]
pub struct ResumeContext {
    pub project_path: String,
    pub last_prompt: String,
    pub files_modified: Vec<String>,
    pub session_date: String,
}

#[tauri::command]
pub async fn open_terminal_at_path(path: String) -> Result<(), String>;
```

**Resume Dialog UI:**
```
┌─────────────────────────────────────┐
│  Resume Session                     │
├─────────────────────────────────────┤
│  📁 Directory:                      │
│  ~/work/my-project           [Copy] │
│                                     │
│  📝 Last Task:                      │
│  Add authentication to the API...   │
│                             [Copy]  │
│                                     │
│  📄 Files Modified:                 │
│  • src/auth/login.rs                │
│  • src/auth/middleware.rs           │
│  • src/api/routes.rs                │
│                                     │
│  [Copy All]  [Open Terminal] [Close]│
└─────────────────────────────────────┘
```

---

### 3.3 Session Actions Menu (Priority: Medium)

**Description:** A unified actions menu on each session item for quick access to all actions.

**UI Design:**
```
┌─────────────────────────────────────┐
│ Session Title                ⋮ (menu)│
│ 📊 47m • 81 messages • 5 files      │
└─────────────────────────────────────┘
            ↓ click menu
┌─────────────────────────────────────┐
│  👁️ View Details                    │
│  📤 Export to Markdown              │
│  🔄 Resume Context                  │
│  📋 Copy Directory                  │
└─────────────────────────────────────┘
```

---

## 🏗️ Technical Implementation

### Backend (Rust)

#### New Commands

```rust
// src-tauri/src/commands/agent_control.rs

use crate::services::session_service::SessionService;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ExportFormat {
    Markdown,
    Json,
}

#[derive(Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub include_full_conversation: bool,
}

/// Export session to specified format
#[tauri::command]
pub async fn export_session(
    session_id: String,
    options: ExportOptions,
) -> Result<String, String> {
    SessionService::export_session(session_id, options)
        .await
        .map_err(|e| e.to_string())
}

/// Get resume context for a session
#[tauri::command]
pub async fn get_resume_context(
    session_id: String,
) -> Result<ResumeContext, String> {
    SessionService::get_resume_context(session_id)
        .await
        .map_err(|e| e.to_string())
}

/// Open terminal at specific path
#[tauri::command]
pub async fn open_terminal_at_path(
    path: String,
) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("osascript")
            .args(["-e", &format!("tell app \"Terminal\" to do script \"cd {}\"", path)])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("gnome-terminal")
            .args(["--working-directory", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", &format!("cd /d {}", path)])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

#### SessionService Updates

```rust
// src-tauri/src/services/session_service.rs

impl SessionService {
    pub async fn export_session(
        session_id: String,
        options: ExportOptions,
    ) -> Result<String, Error> {
        let session = self.get_session(&session_id).await?;
        let messages = self.get_session_messages(&session_id).await?;

        match options.format {
            ExportFormat::Markdown => {
                Ok(self.generate_markdown(&session, &messages, &options))
            }
            ExportFormat::Json => {
                Ok(self.generate_json(&session, &messages, &options))
            }
        }
    }

    fn generate_markdown(
        &self,
        session: &SessionMeta,
        messages: &[Message],
        options: &ExportOptions,
    ) -> String {
        let mut output = String::new();

        // Header
        output.push_str("# Claude Code Session\n\n");

        if options.include_metadata {
            if let Some(stats) = &session.stats {
                writeln!(output, "**Date:** {}", session.created_at.format("%Y-%m-%d %H:%M"));
                writeln!(output, "**Duration:** {} minutes", stats.duration_minutes);
                writeln!(output, "**Project:** {}", session.project_path);
                writeln!(output, "**Model:** {}", stats.model.as_ref().unwrap_or(&"unknown".to_string()));
                output.push_str("\n## Summary\n");
                writeln!(output, "- Messages: {}", stats.message_count);
                writeln!(output, "- Files Modified: {}", stats.files_modified.len());
                writeln!(output, "- Commands Executed: {}", stats.commands_executed.len());
                output.push_str("\n---\n\n");
            }
        }

        // Messages
        if options.include_full_conversation {
            output.push_str("## Conversation\n\n");
            for msg in messages {
                match msg.role.as_str() {
                    "user" => {
                        writeln!(output, "### User: {}", truncate(&msg.content, 100));
                        if options.include_full_conversation {
                            writeln!(output, "{}\n", msg.content);
                        }
                    }
                    "assistant" => {
                        output.push_str("### Assistant Response\n\n");
                        if options.include_full_conversation {
                            writeln!(output, "{}\n", msg.content);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Files and Commands
        if let Some(stats) = &session.stats {
            if !stats.files_modified.is_empty() {
                output.push_str("\n## Files Modified\n");
                for file in &stats.files_modified {
                    writeln!(output, "- `{}`", file);
                }
            }

            if !stats.commands_executed.is_empty() {
                output.push_str("\n## Commands Executed\n");
                for cmd in &stats.commands_executed {
                    writeln!(output, "- `{}`", cmd);
                }
            }
        }

        output
    }

    pub async fn get_resume_context(
        &self,
        session_id: &str,
    ) -> Result<ResumeContext, Error> {
        let session = self.get_session(session_id).await?;
        let messages = self.get_session_messages(session_id).await?;

        // Find last user message
        let last_prompt = messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_else(|| "No prompt found".to_string());

        let files_modified = session
            .stats
            .as_ref()
            .map(|s| s.files_modified.clone())
            .unwrap_or_default();

        Ok(ResumeContext {
            project_path: session.project_path,
            last_prompt,
            files_modified,
            session_date: session.created_at.format("%Y-%m-%d").to_string(),
        })
    }
}
```

### Frontend (TypeScript/React)

#### Type Definitions

```typescript
// src/types.ts

export type ExportFormat = 'markdown' | 'json';

export interface ExportOptions {
  format: ExportFormat;
  includeMetadata: boolean;
  includeFullConversation: boolean;
}

export interface ResumeContext {
  projectPath: string;
  lastPrompt: string;
  filesModified: string[];
  sessionDate: string;
}
```

#### Components

```typescript
// src/components/sessions/SessionItem.tsx

import { useState } from 'react';
import { SessionExportDialog } from './SessionExportDialog';
import { SessionResumeDialog } from './SessionResumeDialog';

export function SessionItem({ session }: { session: SessionMeta }) {
  const [exportOpen, setExportOpen] = useState(false);
  const [resumeOpen, setResumeOpen] = useState(false);

  return (
    <div className="session-item">
      {/* Existing content */}
      <div className="session-actions">
        <button onClick={() => setResumeOpen(true)}>Resume</button>
        <button onClick={() => setExportOpen(true)}>Export</button>
      </div>

      {exportOpen && (
        <SessionExportDialog
          session={session}
          onClose={() => setExportOpen(false)}
        />
      )}

      {resumeOpen && (
        <SessionResumeDialog
          session={session}
          onClose={() => setResumeOpen(false)}
        />
      )}
    </div>
  );
}
```

```typescript
// src/components/sessions/SessionExportDialog.tsx

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { writeFile } from '@tauri-apps/api/fs';

export function SessionExportDialog({
  session,
  onClose,
}: {
  session: SessionMeta;
  onClose: () => void;
}) {
  const [format, setFormat] = useState<ExportFormat>('markdown');
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeFullConversation, setIncludeFullConversation] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [content, setContent] = useState('');

  const handleExport = async () => {
    setExporting(true);
    try {
      const result = await invoke<string>('export_session', {
        sessionId: session.id,
        options: { format, includeMetadata, includeFullConversation },
      });
      setContent(result);
    } catch (error) {
      console.error('Export failed:', error);
    }
    setExporting(false);
  };

  const handleSave = async () => {
    const fileName = `session-${session.id}.${format === 'markdown' ? 'md' : 'json'}`;
    await writeFile(fileName, content);
    onClose();
  };

  const handleCopy = () => {
    navigator.clipboard.writeText(content);
  };

  return (
    <Dialog onClose={onClose}>
      <DialogPanel>
        <DialogTitle>Export Session</DialogTitle>

        <div className="export-options">
          <label>
            <input
              type="radio"
              value="markdown"
              checked={format === 'markdown'}
              onChange={(e) => setFormat(e.target.value as ExportFormat)}
            />
            Markdown
          </label>
          <label>
            <input
              type="radio"
              value="json"
              checked={format === 'json'}
              onChange={(e) => setFormat(e.target.value as ExportFormat)}
            />
            JSON
          </label>

          <label>
            <input
              type="checkbox"
              checked={includeMetadata}
              onChange={(e) => setIncludeMetadata(e.target.checked)}
            />
            Include Metadata
          </label>

          <label>
            <input
              type="checkbox"
              checked={includeFullConversation}
              onChange={(e) => setIncludeFullConversation(e.target.checked)}
            />
            Include Full Conversation
          </label>
        </div>

        <button onClick={handleExport} disabled={exporting}>
          {exporting ? 'Exporting...' : 'Export'}
        </button>

        {content && (
          <div className="export-result">
            <pre>{content}</pre>
            <div className="export-actions">
              <button onClick={handleCopy}>Copy to Clipboard</button>
              <button onClick={handleSave}>Save to File</button>
            </div>
          </div>
        )}
      </DialogPanel>
    </Dialog>
  );
}
```

```typescript
// src/components/sessions/SessionResumeDialog.tsx

import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/shell';

export function SessionResumeDialog({
  session,
  onClose,
}: {
  session: SessionMeta;
  onClose: () => void;
}) {
  const [context, setContext] = useState<ResumeContext | null>(null);

  useEffect(() => {
    invoke<ResumeContext>('get_resume_context', {
      sessionId: session.id,
    }).then(setContext);
  }, [session.id]);

  const copyDirectory = () => {
    navigator.clipboard.writeText(context?.projectPath || '');
  };

  const copyContext = () => {
    const text = `Directory: ${context?.projectPath}\n\nLast Task: ${context?.lastPrompt}`;
    navigator.clipboard.writeText(text);
  };

  const openTerminal = async () => {
    await invoke('open_terminal_at_path', {
      path: context?.projectPath || '',
    });
    onClose();
  };

  if (!context) return <div>Loading...</div>;

  return (
    <Dialog onClose={onClose}>
      <DialogPanel>
        <DialogTitle>Resume Session</DialogTitle>

        <div className="resume-content">
          <div className="resume-section">
            <span className="icon">📁</span>
            <label>Directory:</label>
            <code>{context.projectPath}</code>
            <button onClick={copyDirectory}>Copy</button>
          </div>

          <div className="resume-section">
            <span className="icon">📝</span>
            <label>Last Task:</label>
            <p>{context.lastPrompt}</p>
            <button onClick={copyContext}>Copy Task</button>
          </div>

          {context.filesModified.length > 0 && (
            <div className="resume-section">
              <span className="icon">📄</span>
              <label>Files Modified:</label>
              <ul>
                {context.filesModified.map((file, i) => (
                  <li key={i}>{file}</li>
                ))}
              </ul>
            </div>
          )}
        </div>

        <div className="resume-actions">
          <button onClick={copyContext}>Copy All</button>
          <button onClick={openTerminal}>Open Terminal</button>
          <button onClick={onClose}>Close</button>
        </div>
      </DialogPanel>
    </Dialog>
  );
}
```

---

## 📅 Implementation Checklist

### Backend
- [ ] Add `export_session` command to `agent_control.rs`
- [ ] Add `get_resume_context` command
- [ ] Add `open_terminal_at_path` command
- [ ] Implement markdown export in `SessionService`
- [ ] Implement json export in `SessionService`
- [ ] Implement resume context extraction
- [ ] Add `ResumeContext` struct
- [ ] Add `ExportFormat` and `ExportOptions` structs

### Frontend
- [ ] Create `SessionExportDialog.tsx`
- [ ] Create `SessionResumeDialog.tsx`
- [ ] Add action buttons to `SessionItem.tsx`
- [ ] Update `src/types.ts` with new interfaces
- [ ] Add export/resume translations to i18n files

### Testing
- [ ] Test markdown export format
- [ ] Test json export format
- [ ] Test copy to clipboard
- [ ] Test save to file
- [ ] Test terminal opening on macOS/Linux/Windows
- [ ] Test resume context extraction

---

## 📝 Translation Keys

```json
// src/i18n/locales/en.json
{
  "sessionManager": {
    "export": "Export",
    "resume": "Resume",
    "exportDialogTitle": "Export Session",
    "resumeDialogTitle": "Resume Session",
    "exportFormat": "Export Format",
    "includeMetadata": "Include Metadata",
    "includeFullConversation": "Include Full Conversation",
    "copyToClipboard": "Copy to Clipboard",
    "saveToFile": "Save to File",
    "copyAll": "Copy All",
    "openTerminal": "Open Terminal",
    "directory": "Directory",
    "lastTask": "Last Task",
    "filesModified": "Files Modified"
  }
}

// src/i18n/locales/zh.json
{
  "sessionManager": {
    "export": "导出",
    "resume": "恢复",
    "exportDialogTitle": "导出会话",
    "resumeDialogTitle": "恢复会话",
    "exportFormat": "导出格式",
    "includeMetadata": "包含元数据",
    "includeFullConversation": "包含完整对话",
    "copyToClipboard": "复制到剪贴板",
    "saveToFile": "保存到文件",
    "copyAll": "复制全部",
    "openTerminal": "打开终端",
    "directory": "目录",
    "lastTask": "上次任务",
    "filesModified": "修改的文件"
  }
}
```

---

## 🎯 Success Criteria

- [ ] Users can export any session to Markdown format
- [ ] Export includes metadata, conversation, files, and commands
- [ ] Users can copy exported content or save to file
- [ ] Users can view resume context (directory + last task)
- [ ] Users can open terminal directly in the project directory
- [ ] All actions are accessible from session list items
- [ ] UI is responsive and provides clear feedback
