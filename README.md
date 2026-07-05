下面是一个偏 PRD + Technical Spec 的版本，我按照「真正可以指导开发」的粒度写，而不是只有概念。

FlightDeck —— AI Coding Agent Blackbox Recorder

1. 项目背景

随着 Claude Code、Codex、Gemini CLI、OpenCode 等 AI Coding Agent 的普及，一个新的问题开始出现：

Agent 会做很多事情，但开发者并不知道它到底是如何完成任务的。

例如：

* 为什么修改了这个文件？
* 为什么连续执行了 8 次 npm test？
* 为什么删掉了这个目录？
* 为什么最后失败了？
* 哪一步消耗了最多时间？
* 哪一步最容易失败？
* 不同模型解决同一个问题时有什么区别？

目前，大多数 Agent 都只保留聊天记录，而真正重要的执行过程（Tool、Shell、File、Diff、Hook、状态变化）几乎没有统一的可视化。

FlightDeck 希望成为：

AI Coding Agent 的 Flight Recorder（飞行记录仪）

类似飞机黑匣子一样，完整记录一次 Agent Session 的全过程，并提供可观测、可分析、可分享的能力。

⸻

2. 项目目标

FlightDeck 不负责：

* 推理
* 写代码
* 替代 Claude Code
* 替代 Codex

FlightDeck 只负责：

Observe（观察）

记录 Agent 的全部行为。

整个架构属于：

Developer
      │
      ▼
Claude Code / Codex
      │
      ▼
 FlightDeck
      │
      ▼
 Local Database
      │
      ▼
 Dashboard

⸻

3. MVP

第一版只实现最核心能力。

Session

自动开始记录：

Session Started
Project:
/Users/xxx/project
Model:
Claude Sonnet
Branch:
feature/login
Started:
09:15

结束后：

Duration:
18m
Events:
132
Files Changed:
9
Commands:
24
Failures:
2

⸻

Timeline

时间轴展示所有事件。

09:15 User Prompt
09:15 Read README.md
09:16 Read package.json
09:17 Edit auth.ts
09:18 npm test
09:19 Test Failed
09:20 Edit auth.ts
09:22 npm test
09:23 Success
09:24 Session End

类似 Chrome DevTools Performance Timeline。

⸻

Command Recorder

记录：

Command
npm test
Duration
34.2s
Exit Code
1
stdout
...
stderr
...

可以折叠查看。

⸻

File Recorder

记录：

Read
src/api.ts
Write
src/auth.ts
Delete
tmp/cache.json
Rename
...

并支持：

Git Diff

直接展示修改内容。

⸻

Error Recorder

如果：

Exit Code != 0

自动标红：

❌ npm test
Reason
TypeError
Duration
18s

⸻

Metrics

统计：

Total Files
24
Commands
31
Failures
3
Retry
6
Estimated Tokens
...
Execution Time
18m

⸻

Markdown Export

自动生成：

# AI Session Report
Prompt
...
Timeline
...
Files Changed
...
Commands
...
Errors
...
Summary

方便分享到 GitHub。

⸻

4. 第二阶段

Replay

点击：

Replay

重新播放整个 Session。

像录像一样：

Read
↓
Edit
↓
Command
↓
Fail
↓
Retry
↓
Success

⸻

AI Analysis

调用 LLM：

Analyze This Session

自动生成：

Problems
- 重复运行测试
- 多次读取同一文件
- Context Drift
Suggestions
...

相当于：

AI Review AI。

⸻

Session Compare

比较两个 Session：

Yesterday
Today

例如：

Execution Time
12m
↓
6m
Failures
5
↓
1
Commands
34
↓
18

⸻

5. 第三阶段

支持：

* Claude Code
* Codex
* Gemini CLI
* OpenCode
* Aider
* Cline
* Continue

统一抽象：

Agent Event

例如：

interface Event {
type
timestamp
payload
}

所有 Agent 最终转换成统一事件流。

⸻

6. Event Schema

统一定义：

SessionStarted
SessionEnded
Prompt
ReadFile
WriteFile
DeleteFile
RenameFile
ShellStart
ShellEnd
ToolCall
ToolResult
TestPassed
TestFailed
Error
Warning
GitCommit
Notification

所有 Dashboard 都依赖这一层。

⸻

7. Dashboard

左侧：

Sessions

中间：

Timeline

右侧：

Inspector

类似：

Chrome DevTools

点击：

npm test

右侧展示：

stdout
stderr
exit code
duration

点击：

Edit auth.ts

展示：

Git Diff

⸻

8. 技术架构

Claude Hook
↓
Event Adapter
↓
Local Collector
↓
SQLite
↓
Backend API
↓
React Dashboard

推荐：

Collector

Go

Dashboard

React
Vite
TailwindCSS

数据库

SQLite
DuckDB（后期）

事件协议

JSONL

例如：

{
  "timestamp": "...",
  "type": "ShellEnd",
  "duration": 28,
  "exitCode": 1
}

⸻

9. 插件系统

每个 Agent 一个 Adapter：

Adapter
Claude
Codex
Gemini
OpenCode
...

统一接口：

Start()
Stop()
Parse()
Emit()

以后增加新的 Agent 不影响核心。

⸻

10. 项目亮点

Local First

所有数据默认保存在本地。

无需云端。

⸻

Privacy First

默认：

No Upload

所有分析都可以本地完成。

⸻

Open Protocol

开放：

Agent Event Protocol

任何 Agent 都可以接入。

⸻

Share

可以生成：

session.flightdeck

别人导入即可 Replay。

⸻

11. 后续演进

如果项目发展顺利，可以逐步扩展：

FlightDeck Cloud

团队共享 Session。

⸻

FlightDeck Arena

多个 Agent 同时执行一个任务，对比：

* 耗时
* 修改文件数
* Shell 命令数
* 成功率
* Retry 次数
* Token 消耗

⸻

FlightDeck Observatory

多 Agent 实时监控：

Claude
● Coding
Codex
● Testing
Gemini
● Reading Docs
OpenCode
● Refactoring

类似一个实时控制中心。

⸻

12. 一句话定位（Pitch）

FlightDeck 是 AI Coding Agent 的飞行记录仪。它通过统一的事件协议记录 Claude Code、Codex 等 Agent 的完整执行过程，让开发者能够观察、分析、回放和优化 AI 编程工作流，而无需修改 Agent 本身。

我觉得这个方向还有一个可以做得更有影响力的点：不要把它做成一个产品，而是先做成一个标准。

也就是说，先定义一套 Agent Event Protocol（AEP），规定 AI Coding Agent 如何描述 ReadFile、ShellStart、ToolCall、PatchApplied、SessionEnd 等事件。FlightDeck 只是这个协议的第一个实现。

如果 AEP 被更多 Agent 或工具采用，那么以后任何可视化、分析、回放、安全审计工具都能基于同一套事件流工作。相比单纯做一个 Recorder，这种协议层的价值和影响力会更大。
