# Agent Control Panel for Claude Code - Implementation Plan

## 项目概述
基于 cc-switch 项目开发一个桌面应用，用于观察、分析并持续优化本地 Claude Code Agent 的行为。该应用将作为非侵入式的增强层，监听 `~/.claude` 目录中的会话数据，而不需要修改 Claude Code 本身的运行时。

## 架构概览
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

## 实施步骤

### Phase 1: 基础框架搭建 (Day 1-2)
#### Step 1: 环境准备和项目初始化
- [ ] 在 src-tauri/src/ 目录下创建必要的目录结构
- [ ] 创建 `watcher/` 目录及其模块文件
- [ ] 创建 `parser/` 目录及其模块文件
- [ ] 创建 `services/` 目录中的新服务文件
- [ ] 更新 Cargo.toml 添加必要的依赖 (如 notify crate 用于文件监听)

#### Step 2: Claude Watcher 实现
- [ ] 创建 `src-tauri/src/watcher/claude_watcher.rs`
- [ ] 实现基本的文件系统监听功能
- [ ] 监听 `~/.claude/sessions/` 目录的新建和修改事件
- [ ] 添加防抖机制 (500ms)，避免事件风暴
- [ ] 创建事件总线机制用于通知其他组件

### Phase 2: 数据处理层 (Day 3-5)
#### Step 3: Session Parser 实现
- [ ] 创建 `src-tauri/src/parser/session_parser.rs`
- [ ] 定义 Session 结构体，包含 id、workspace_path、start_time、prompt、commands[]、files[]、summary 等字段
- [ ] 实现解析 Claude session JSON 文件的功能
- [ ] 提取关键信息：prompt、timestamp、workspace、messages、commands、files
- [ ] 错误处理机制，应对 session 格式变化的风险

#### Step 4: 数据库 Schema 设计和实现
- [ ] 在数据库 schema 中添加新的表结构：
  - sessions 表 (id, workspace, prompt, start_time, file_count, command_count)
  - commands 表 (id, session_id, command)
  - files 表 (id, session_id, path)
  - memory 表 (id, workspace, key, value, confidence, updated_at)
  - prompts 表 (id, text, count)
- [ ] 更新 migration.rs 添加新的数据库迁移脚本
- [ ] 在 dao/ 目录下创建相应的数据访问对象

#### Step 5: 后端服务层实现
- [ ] 创建 `src-tauri/src/services/session_service.rs`
- [ ] 创建 `src-tauri/src/services/memory_service.rs`
- [ ] 创建 `src-tauri/src/services/insight_service.rs`
- [ ] 实现将解析的会话数据存储到数据库的功能
- [ ] 实现基础的会话查询 API

### Phase 3: 智能引擎开发 (Day 6-9)
#### Step 6: Memory Engine v1 实现
- [ ] 在 memory_service.rs 中实现记忆提取算法
- [ ] 统计算法：识别高频命令（如 pytest、poetry、docker）
- [ ] 建立工作区级别的记忆存储
- [ ] 设置置信度评分机制
- [ ] 实现记忆的自动更新机制

#### Step 7: Insight Engine 实现
- [ ] 在 insight_service.rs 中实现分析功能
- [ ] 实现会话统计查询（按工作区、时间段等）
- [ ] 实现命令使用频率统计
- [ ] 实现提示词使用频率统计
- [ ] 实现文件/目录编辑频率统计

### Phase 4: 前端界面开发 (Day 10-14)
#### Step 8: 前端路由和页面结构
- [ ] 在 src/ 目录下创建新的页面组件：
  - pages/WorkspacesPage.tsx
  - pages/SessionsPage.tsx
  - pages/MemoryPage.tsx
  - pages/InsightsPage.tsx
  - pages/ErrorsPage.tsx
- [ ] 在主应用中注册新的路由
- [ ] 设计统一的导航结构

#### Step 9: 会话时间线页面
- [ ] 实现会话列表展示功能
- [ ] 显示会话的 prompt、时间戳、工作区、编辑文件、执行命令等信息
- [ ] 实现会话详情查看功能
- [ ] 实现按日期分组的时间线视图

#### Step 10: 工作区页面
- [ ] 实现工作区自动识别和分组
- [ ] 显示每个工作区的活跃度指标
- [ ] 实现工作区间的切换功能
- [ ] 显示工作区内的历史会话

#### Step 11: 记忆页面
- [ ] 展示项目级记忆（首选工具、测试命令、包管理器等）
- [ ] 提供手动编辑记忆的功能
- [ ] 显示记忆的置信度和最后更新时间

#### Step 12: 洞察页面
- [ ] 实现仪表板布局
- [ ] 展示使用统计图表（会话数量、任务类型分布等）
- [ ] 显示高频命令和提示词列表
- [ ] 显示常用编辑的文件夹

### Phase 5: 集成与优化 (Day 15-16)
#### Step 13: 系统集成
- [ ] 连接监听器、解析器、服务层和数据库
- [ ] 实现前端到后端的完整数据流
- [ ] 测试事件总线机制，确保前端能及时更新
- [ ] 修复集成过程中的问题

#### Step 14: 性能优化和稳定性
- [ ] 优化大数据量下的性能（大量会话的情况）
- [ ] 添加日志记录功能便于调试
- [ ] 实现错误恢复机制
- [ ] 测试长时间运行的稳定性

### Phase 6: 测试和完善 (Day 17-20)
#### Step 15: 功能测试
- [ ] 测试监听器能否正确捕获新的 Claude 会话
- [ ] 测试解析器能否正确处理各种会话格式
- [ ] 测试前端页面的数据展示准确性
- [ ] 测试记忆引擎的统计准确性

#### Step 16: 用户体验完善
- [ ] 优化界面交互
- [ ] 添加加载状态和错误提示
- [ ] 完善帮助文档
- [ ] 进行可用性测试

## 技术要点
1. **非侵入性设计**: 严格遵守只读原则，不对 `~/.claude/sessions/` 或 `~/.claude/logs/` 目录进行任何写入操作
2. **内存引擎**: 基于统计的简单算法而非复杂AI模型
3. **数据安全**: 所有记忆数据存储在 `~/.cc-switch/cc-switch.db` 中，不污染 Claude 环境
4. **兼容性**: 具备应对 Claude session 格式变化的能力

## 风险评估及对策
1. **Claude session 格式变化**: 实现解析器版本控制和向后兼容
2. **Session 文件过大**: 仅解析元数据，不加载完整对话内容
3. **Watcher 事件风暴**: 实现防抖机制，限制处理频率
4. **数据库性能**: 适时引入索引和分页机制

## MVP 交付物 (预计 Day 1-10)
- Claude 会话时间线展示
- 工作区分组功能
- 提示词历史记录
- 基础记忆提取功能