# FlightDeck

AI Coding Agent 的飞行记录仪。记录 Claude Code、Codex 等 Agent 的完整执行过程。

## 快速开始

```bash
# 启动 API 服务
cargo run --bin flightdeck-api

# 启动 Dashboard（另一个终端）
cd dashboard && npm install && npm run dev
```

访问 http://localhost:3000

## 项目结构

```
crates/
├── flightdeck-core/       # 事件协议定义
├── flightdeck-storage/    # SQLite 存储
├── flightdeck-collector/  # 事件收集框架
├── flightdeck-api/        # HTTP API
└── flightdeck-adapters/   # Agent 适配器
dashboard/                 # React 前端
```

## API

| 端点 | 说明 |
|------|------|
| `GET /api/health` | 健康检查 |
| `GET /api/sessions` | 会话列表 |
| `GET /api/sessions/:id` | 会话详情 |
| `GET /api/sessions/:id/events` | 事件列表 |
| `GET /api/metrics` | 统计指标 |

## 技术栈

- **后端**: Rust + Axum + SQLite
- **前端**: React + Vite + TailwindCSS
- **协议**: Agent Event Protocol (AEP)

## License

MIT