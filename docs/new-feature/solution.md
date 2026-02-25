下面是一份基于你 PRD 的 **工程级技术方案（Technical Design Doc v1）**，专门针对：

* 只支持 Claude Code
* Fork **cc-switch** 作为基础
* 方案B：Observer Architecture（监听 ~/.claude）
* Desktop GUI
* 本地 Memory Engine
* 非侵入

这份方案是可以直接进入开发阶段的。

---

# Technical Design v1

## 项目名称（暂定）

**Agent Control Panel for Claude Code**

副标题：

> Observability & Evolution Layer for Claude Code

---

# 1. 基础技术选择

## 1.1 Fork 基础项目

基础 repo：

* cc-switch

这是一个跨平台桌面应用，支持 Claude Code 配置管理。 ([GitHub][1])

已经包含：

* Tauri Desktop framework
* React UI
* SQLite 数据层
* ConfigService
* ProviderService
* settings.json 写入逻辑

例如：

Claude 配置：

```
~/.claude/settings.json
```

已经被 cc-switch 支持读写。 ([GitHub][1])

---

## 1.2 技术栈

直接继承 cc-switch：

### Desktop

```
Tauri 2.x
Rust backend
```

原因：

* 已经实现
* 小内存
* native性能
* filesystem watcher方便

---

### Frontend

```
React
TypeScript
TanStack Query
Tailwind
```

cc-switch 已经实现：

```
Components
Hooks
Services
```

---

### Storage

cc-switch 已经有：

```
~/.cc-switch/cc-switch.db
```

SQLite 单一真相源（SSOT）。 ([GitHub][1])

直接复用。

只扩展 schema。

---

# 2. 系统架构

这是最终结构：

```
Claude Code (~/.claude)
          ↓

ClaudeWatcher (NEW)
          ↓

SessionParser (NEW)
          ↓

Event Bus (NEW)
          ↓

Services Layer

- SessionService (NEW)
- MemoryService (NEW)
- InsightService (NEW)

          ↓

SQLite

          ↓

React GUI
```

---

# 3. 与 cc-switch 的关系

cc-switch 当前结构：

```
Commands
Services
DAO
Database
```

你的扩展：

```
Commands
Services
DAO
Database

+ Watcher Layer
+ Parser Layer
+ Intelligence Layer
```

建议：

```
src-tauri/src/

services/
    session_service.rs
    memory_service.rs
    insight_service.rs

watcher/
    claude_watcher.rs

parser/
    session_parser.rs
```

---

# 4. Claude 数据源设计

只依赖：

```
~/.claude/
```

包括：

```
sessions/
logs/
settings.json
skills/
CLAUDE.md
```

---

# 5. Claude Watcher

这是系统核心。

---

## 5.1 监听目标

监听：

```
~/.claude/sessions/
```

检测：

* 新session
* session更新

Rust建议：

```
notify crate
```

结构：

```
ClaudeWatcher
   start()
   on_file_created()
   on_file_modified()
```

---

## 5.2 Watcher流程

```
New Session File

→ Parse Session

→ Insert DB

→ Update Memory

→ Update Insights

→ Emit Event
```

---

## 6. Session Parser

模块：

```
parser/session_parser.rs
```

---

## 6.1 输入

Session JSON：

例如：

```
~/.claude/sessions/abc123.json
```

解析：

```
prompt
timestamp
workspace
messages
commands
files
```

---

## 6.2 输出结构

Rust struct：

```
Session {

 id
 workspace_path

 start_time

 prompt

 commands[]

 files[]

 summary

}
```

---

## 7. Database Schema

扩展 SQLite。

cc-switch 已经有 DB migration 系统。

可以直接加 schema version。

---

## 7.1 Sessions Table

```
sessions

id TEXT
workspace TEXT

prompt TEXT

start_time INTEGER

file_count INTEGER

command_count INTEGER
```

---

## 7.2 Commands Table

```
commands

id
session_id

command TEXT
```

---

## 7.3 Files Table

```
files

id
session_id

path TEXT
```

---

## 7.4 Memory Table

核心：

```
memory

id

workspace

key

value

confidence

updated_at
```

示例：

```
workspace:

my-app

key:

test_command

value:

pytest

confidence:

0.82
```

---

## 7.5 Prompt Table

```
prompts

id

text

count
```

用于 prompt analytics。

---

# 8. Memory Engine

核心智能模块。

---

## 8.1 输入

来自：

```
sessions table
commands table
files table
prompts table
```

---

## 8.2 输出

生成：

```
memory table
```

例如：

```
Preferred tools:

pytest
poetry
docker
```

---

## 8.3 算法 v1

不要用 AI。

只用统计。

例如：

```
top command:

pytest
```

规则：

```
frequency > 5

→ memory entry
```

---

示例：

```
commands:

pytest x 12
npm x 2
```

生成：

```
test_command = pytest
```

---

# 9. Insight Engine

生成：

```
Insights Dashboard
```

---

## 9.1 指标

统计：

```
sessions per workspace

top commands

top prompts

top folders
```

SQL就够。

例如：

```
SELECT command, COUNT(*)
FROM commands
GROUP BY command
ORDER BY COUNT DESC
```

---

# 10. Event Bus

用于实时更新 GUI。

---

结构：

```
Watcher

→ emit session_added

Frontend refresh
```

Tauri：

```
app.emit_all("session-added")
```

---

# 11. Frontend Architecture

扩展 cc-switch。

建议新增：

```
pages/

WorkspacesPage.tsx
SessionsPage.tsx
MemoryPage.tsx
InsightsPage.tsx
ErrorsPage.tsx
```

---

# 12. MVP 实现顺序（非常关键）

推荐顺序：

---

## Step 1

ClaudeWatcher

目标：

```
检测新session
```

时间：

1天

---

## Step 2

SessionParser

目标：

```
Session list UI
```

时间：

2天

---

## Step 3

Workspace grouping

目标：

```
Workspaces page
```

时间：

1天

---

## Step 4

Prompt history

目标：

```
Top prompts
```

时间：

1天

---

## Step 5

Memory Engine v1

目标：

```
Project memory
```

时间：

3–4天

---

总计：

```
~10天
```

可以出 v1。

---

# 13. 推荐代码结构（最终形态）

```
src-tauri/

services/

  provider_service.rs
  config_service.rs

  session_service.rs
  memory_service.rs
  insight_service.rs


watcher/

  claude_watcher.rs


parser/

  session_parser.rs


db/

  migrations/
```

---

# 14. 关键技术决策（非常重要）

## 决策1：只读 ~/.claude

永远不修改：

```
sessions/
logs/
```

只允许写：

```
settings.json
CLAUDE.md
```

复用 cc-switch。

安全。

---

## 决策2：Memory独立存储

不要：

```
~/.claude/memory.json
```

而是：

```
~/.cc-switch/cc-switch.db
```

原因：

* 不污染Claude
* 可迁移
* 可版本化

---

## 决策3：Observer-first

所有feature：

必须可通过：

```
passive observation
```

实现。

---

# 15. v1 技术风险

## 风险1

Claude session格式变化。

解决：

```
parser versioning
```

---

## 风险2

Session文件过大。

解决：

只解析metadata。

---

## 风险3

Watcher事件风暴。

解决：

debounce：

```
500ms
```

---

# 16. v1 结束状态

完成后你会有：

* Claude Session Timeline
* Workspace grouping
* Prompt analytics
* Basic memory extraction

这已经是：

> 世界上第一个 Claude Observability Desktop Tool

而且：

cc-switch作为基础是一个**非常正确的工程选择**：

* SQLite
* Tauri
* Config writing
* Prompt presets

这些已经帮你省掉至少 **3–4周工程时间**。

---

如果你愿意，我可以帮你设计一个 **极其关键的模块：Session Parser Spec（Claude sessions JSON 精确结构 + parser代码设计）** ——这是整个系统最核心的一层。

[1]: https://github.com/farion1231/cc-switch?utm_source=chatgpt.com "GitHub - farion1231/cc-switch: A cross-platform desktop All-in-One assistant tool for Claude Code, Codex & Gemini CLI."
