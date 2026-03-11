# M1 Audit Query Stage Kickoff

## 日期

2026-03-11

## 同步目的

在进入 `M1-W08 Audit Replay Query` 前，明确当前版本、GitHub 基线和本阶段的交付边界，保证“审计回放能力”作为正式阶段推进，而不是零散补接口。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`audit replay and filtered audit queries`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`8ec5bd2`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- `FileTaskRepository`
- `FileAuditStore`
- `FA_DATA_DIR` 本地持久化运行模式
- 真实服务重启后的任务与审计回读验证

## 本阶段目标

当前系统已经能记录审计事件，但在接口层仍主要是“全量取回”。这不利于试运行、运维排障和任务复盘。

本阶段目标：

1. 增加按任务过滤的审计回放能力。
2. 增加按 `correlation_id` 和事件类型过滤的审计查询能力。
3. 保持内存模式和文件模式下的行为一致。
4. 为后续 audit replay view / 运维查询打基础。

## 本阶段交付边界

本阶段计划交付：

- audit query model
- audit store 查询抽象
- `/api/v1/audit/events` 过滤能力
- `/api/v1/tasks/{task_id}/audit-events` 回放接口
- service-level query tests
- 文档同步

本阶段暂不交付：

- 分页与排序策略
- 图形化审计回放页面
- 审计事件跨任务聚合分析

## 风险与注意事项

- 如果只增加查询参数而没有固定任务回放入口，用户仍然不容易定位单任务历史。
- 如果现在就引入复杂检索能力，会拖慢 `M1` 节奏。
- 必须保证文件模式与内存模式下的审计查询结果一致。

## 进入本阶段的理由

有了持久化之后，下一步不应该只“存下来”，而应该“找得到、看得懂、能复盘”。审计回放能力是平台从“会记住”走向“可运营”的关键一步。

## 当前推进结果

本阶段已经完成：

- `AuditEventQuery`
- audit store 查询抽象
- `/api/v1/audit/events` 过滤能力
- `/api/v1/tasks/{task_id}/audit-events` 回放接口
- audit query 单元测试
- service-level 回放与过滤测试
- 文档同步

## 当前验证结果

已完成的质量验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- 在 `FA_DATA_DIR=/tmp/fa-audit-query-smoke.Fshmdn` 下完成真实 HTTP smoke test

运行态已确认：

- `/api/v1/audit/events?correlation_id=audit-query-smoke-002` 返回精确的审批链路事件
- `/api/v1/tasks/99c3ab89-4f67-4fdb-b0c8-4c717226ee81/audit-events` 返回单任务完整审计回放
- `/api/v1/audit/events?task_id=99c3ab89-4f67-4fdb-b0c8-4c717226ee81&kind=approval_requested` 返回精确过滤结果

当前结论：

- 审计能力已经从“全量导出”进入“按业务主键回放和定位”
- `task_id` 和 `correlation_id` 现在已经成为真实可用的运行管理入口
- 下一步可以在这个基础上构建审计回放视图和更强查询语义
