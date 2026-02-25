# Agent Control Panel for Claude Code - Implementation Plan (Simplified)

**Updated:** 2026-02-25
**Focus:** Claude Code only, simple features first

---

## 项目概述
基于 cc-switch 项目开发一个轻量级桌面应用，用于观察和管理 Claude Code 会话。非侵入式监听 `~/.claude` 目录，不修改 Claude Code 运行时。

## 架构概览
```
Claude Code (~/.claude)
          ↓
AgentWatcher (NEW)
          ↓
SessionParser (Claude only)
          ↓
SessionService (NEW)
          ↓
SQLite Database
          ↓
React GUI (SessionManagerPage)
```

---

## 实施步骤

### Phase 1: 基础监听 ✅ DONE
#### Agent Watcher
- [x] 创建 `src-tauri/src/services/agent_watcher.rs`
- [x] 监听 `~/.claude/projects/**/*.jsonl` 文件
- [x] 添加防抖机制 (500ms)

#### Session Parser (Claude Only)
- [x] 创建 `src-tauri/src/parser/` 模块
- [x] 解析 Claude session 格式
- [x] 提取元数据：duration, messages, tools, files, commands

#### Session Service
- [x] 创建 `src-tauri/src/services/session_service.rs`
- [x] 会话存储和查询 API

### Phase 2: 前端界面 ✅ DONE
#### Session Manager Page
- [x] 会话列表展示 (SessionItem 组件)
- [x] 日期分组 (Today, Yesterday, This Week, Older)
- [x] 统计徽章 (SessionStatsBadge: duration, messages, files, commands)

### Phase 3: 增强搜索 ✅ DONE (2026-02-25)
#### 搜索功能
- [x] 按文件名搜索会话
- [x] 按命令搜索会话
- [x] 按工具使用筛选
- [x] **Bonus:** 关键词搜索 (标题、摘要、会话 ID)
- [x] **Bonus:** 项目目录筛选

#### 后端支持
```rust
// src-tauri/src/commands/session_manager.rs
pub async fn search_sessions(query: SessionSearchQuery) -> Result<SessionSearchResult, String>;
```

#### 前端组件
```typescript
// src/components/sessions/
- SessionSearchBar.tsx        ✅ 高级搜索栏 (5 种过滤器)
- useBackendSessionSearch.ts  ✅ 后端搜索 hook
```

#### 搜索结果
- 匹配的文件列表
- 匹配的命令列表
- 匹配的工具列表
- 按时间排序

### Phase 4: 快捷操作 (下一步)
#### 导出功能
- [ ] 导出会话为 Markdown
- [ ] `export_session(id: String) -> Result<String>`

#### 快速恢复
- [ ] 复制工作目录 + 上下文
- [ ] `get_resume_context(id: String) -> Result<ResumeContext>`

---

## 数据库 Schema (简化版)

```sql
-- 会话表
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_path TEXT,
    created_at TEXT,
    last_active_at TEXT,
    duration_minutes INTEGER,
    message_count INTEGER,
    tools_used TEXT,        -- JSON: ["Write", "Read", "Bash"]
    files_modified TEXT,    -- JSON: ["src/main.rs"]
    commands_executed TEXT, -- JSON: ["/clear", "commit"]
    model TEXT,
    topic TEXT
);
```

---

## 技术要点
1. **Claude Code Only** - 仅支持 Claude Code，不支持其他 CLI 工具
2. **非侵入性** - 只读 `~/.claude`，不写入
3. **简单优先** - 核心功能优先，暂不实现复杂分析
4. **本地存储** - 数据存储在应用数据库中

---

## 风险及对策
| 风险 | 对策 |
|------|------|
| Session 格式变化 | 版本化解析器，向后兼容 |
| 文件过大 | 仅解析元数据，不加载完整内容 |
| 性能问题 | 分页查询，虚拟滚动 |

---

## 交付状态

### ✅ 已完成
- Agent Watcher 文件监听
- Claude Session Parser
- Session Service 后端
- Session Manager Page 前端
- 日期分组 UI
- 统计徽章组件
- **增强搜索 (文件/命令/工具/关键词/项目目录)**

### ⏳ 下一步
- 导出功能 (Markdown)
- 快速恢复 (工作目录 + 上下文)
