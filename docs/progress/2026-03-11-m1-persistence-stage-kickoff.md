# M1 Persistence Stage Kickoff

## 日期

2026-03-11

## 同步目的

在进入 `M1-W07 Local Persistence Baseline` 前，先明确当前版本状态、远端基线和本阶段交付边界，避免“存储演进”在没有治理记录的情况下直接侵入运行时。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`local durable storage for tasks and audit events`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`d0ad77f`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- revision resubmission loop
- `POST /api/v1/tasks/{task_id}/resubmit`
- 驳回后非法执行拦截
- 修订闭环的服务层与运行态验证

## 本阶段目标

当前系统已经具备受治理的任务闭环，但任务状态和审计事件默认仍停留在内存。只要服务重启，运行历史就会丢失，这不适合后续试运行准备。

本阶段目标：

1. 增加本地文件型 task repository。
2. 增加本地文件型 audit store。
3. 通过环境变量让 `fa-server` 可切换到本地耐久模式。
4. 验证服务重启后仍能回读任务和审计记录。

## 本阶段交付边界

本阶段计划交付：

- file-backed task repository
- file-backed audit store
- `FA_DATA_DIR` 运行时注入能力
- 文件型持久化测试
- 重启后回读的 smoke test
- 文档同步

本阶段暂不交付：

- SQLite / Postgres
- 并发锁和事务语义
- 历史版本 diff 查询

## 风险与注意事项

- 本地文件持久化只能作为基线，不应伪装成企业级数据库能力。
- 必须保持现有内存模式可用，避免把日常开发全部绑到文件模式。
- 审计和任务存储需要一起落地，否则重启后只能恢复部分上下文。

## 进入本阶段的理由

如果平台一重启就失忆，它就还不具备试运行前的最低可信度。先把本地持久化打通，是为了让系统从“可运行原型”继续走向“可持续运行原型”。

## 当前推进结果

本阶段已经完成：

- `FileTaskRepository`
- `FileAuditStore`
- `FA_DATA_DIR` 运行时注入能力
- 文件型持久化单元测试
- 真实服务重启回读 smoke test
- 文档同步

## 当前验证结果

已完成的质量验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- 在 `FA_DATA_DIR=/tmp/fa-persist-smoke.LbGSBK` 下完成真实服务写入、重启和回读

运行态已确认：

- 任务文件已写入 `tasks/e9122057-e68e-47f5-b05c-32420b85324b.json`
- 审计文件已写入 `audit-events.jsonl`
- 服务重启后 `GET /api/v1/tasks/{task_id}` 能回读已写入任务
- 服务重启后 `/api/v1/audit/events` 能回读已写入审计记录

当前结论：

- FA 已经不再只依赖进程内存保存状态
- 本地试运行准备具备了最低耐久性基线
- 下一步可以把同样边界演进到更强的持久化实现
