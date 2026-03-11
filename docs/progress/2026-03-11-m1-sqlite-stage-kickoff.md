# M1 SQLite Stage Kickoff

## 日期

2026-03-11

## 同步目的

在进入 `M1-W09 SQLite Baseline` 前，正式同步当前版本、GitHub 基线和本阶段范围，确保 SQLite 基线不是零散的本地实验，而是可追溯的正式交付阶段。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`sqlite-backed local persistence`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`147e43c`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- audit replay query baseline
- `/api/v1/audit/events` 多维过滤
- `/api/v1/tasks/{task_id}/audit-events` 单任务回放
- 文件模式下真实 HTTP 审计查询验证

## 本阶段目标

当前系统已经支持内存模式和文件模式，但还缺少一个更接近数据库形态的本地结构化存储基线。SQLite 是当前最合理的下一步。

本阶段目标：

1. 增加 SQLite-backed task repository。
2. 增加 SQLite-backed audit store。
3. 通过环境变量让 `fa-server` 切换到 SQLite 模式。
4. 验证任务、审计和审计查询在 SQLite 模式下成立。

## 本阶段交付边界

本阶段计划交付：

- `SqliteTaskRepository`
- `SqliteAuditStore`
- `FA_SQLITE_DB_PATH` 运行时注入能力
- SQLite 单元测试
- SQLite 模式下真实 HTTP smoke test
- 文档同步

本阶段暂不交付：

- ORM / migration 框架
- 跨进程高并发写入策略
- 数据库连接池和数据库级权限体系

## 风险与注意事项

- 当前阶段优先建立本地 SQLite 基线，不追求一次到位的企业级数据库能力。
- 实现必须保持内存模式、文件模式和 SQLite 模式并存，避免破坏当前开发流。
- 审计查询能力在 SQLite 模式下也必须成立，不能只保证存储。

## 进入本阶段的理由

文件模式解决了“重启不失忆”，但数据库模式才能更自然地承接结构化查询、后续迁移和更强的持久化路线。SQLite 基线是从原型走向更真实运行环境的自然下一步。

## 本阶段完成结果

- 已交付 `SqliteTaskRepository`
- 已交付 `SqliteAuditStore`
- `fa-server` 已支持 `FA_SQLITE_DB_PATH` 运行时注入
- 持久化注入顺序已明确为 `SQLite -> File -> Memory`
- SQLite 模式下的任务读取、审计写入、审计过滤查询和任务审计回放均已成立

## 实现摘要

本阶段采用务实路线，先通过本机 `sqlite3` CLI 建立 SQLite 基线，而不是等待更重的数据库依赖和迁移框架到位。

这样做的结果是：

- 平台获得了结构化、本地可部署、可重启回读的数据库基线
- API 与编排器不需要感知底层存储变化
- 后续从 SQLite 走向更强数据库后端时，边界已经被抽象清楚

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`

已完成真实 HTTP smoke test：

- 使用 `FA_SERVER_ADDR=127.0.0.1:8000`
- 使用独立数据库文件 `/tmp/fa-sqlite-smoke-1773243368-74861.db`
- 创建任务 `3fc07816-17c6-4ff8-9f56-5f1b3875218d`
- 关联链路 `sqlite-smoke-001`
- 停止并重启服务后，任务详情与 `/api/v1/audit/events?task_id=...` 审计历史均成功回读

## 阶段收口结论

SQLite 基线已经满足本阶段目标，但边界也保持清楚：

- 它是本地结构化持久化基线，不是最终企业级数据库方案
- 它优先解决耐久性、查询承载和迁移路径问题
- 下一阶段应把重点切回 pilot workflow 选择、规格冻结和发布准备
