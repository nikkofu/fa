# M1 Follow-up Owner Queue Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `follow-up owner acceptance` 已经成立后，继续把高频协同闭环从“单任务里谁接手了”推进到“跨任务看谁手里有哪些待办”。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up owner queue`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `follow_up_items` 从 task-scoped detail 推进到最小 cross-task owner queue read

## 上一阶段完成基线

上一阶段已完成：

- `shift handoff` 与 `alert triage` 已能稳定生成 seeded `follow_up_item`
- `follow-up-items/{follow_up_id}/accept-owner` 已能表达谁真正接手了 item
- 下一层最值得推进的是跨任务 owner queue，而不是继续堆新的 task detail 字段

## 本阶段目标

1. 增加最小 `GET /api/v1/follow-up-items` cross-task owner queue read。
2. 先复用 repository 中已持久化的 `TrackedTaskState` 扫描，不引入新 projection 表。
3. 支持 `task_id / source_kind / status / owner_id / owner_role / overdue_only` 最小过滤。
4. 用默认排序体现现场优先级，并同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- repository `list()` 基线在内存 / 文件 / SQLite 模式下可复用
- `WorkOrchestrator::list_follow_up_items(...)`
- `GET /api/v1/follow-up-items`
- orchestrator / API / sandbox-safe file mode 测试覆盖
- 项目状态同步

本阶段暂不交付：

- dedicated follow-up projection 表
- `blocked_only / risk / priority / due_before` 等扩展过滤
- follow-up item completion / blocked / reassign action
- 独立 overdue monitor 或 escalation worker

## 风险与注意事项

- 不能把 queue API 误做成新的写模型或 assignment 系统
- 不能把 accepted owner 和 recommended owner 混成同一字段语义
- 不能把 audit replay 当成 backlog query 主数据源

## 进入本阶段的理由

如果系统只能在单个任务里回答“这条 follow-up 被谁接手了”，但不能跨任务回答“这个角色现在手里有哪些待办”，那它仍然离高频运营协同很远。先补最小 queue read，能最快验证现有 repository、task detail 和过滤模型是否足够支撑第一层 owner queue。

## 本阶段完成结果

- 已为 `follow_up_items` 增加最小 cross-task owner queue read API
- 已验证 queue 会聚合 `shift handoff` 与 `alert triage` 的 follow-up item
- 已验证 `owner_id / source_kind` 过滤成立
- 已验证 file-backed sandbox 模式重启后仍可回读 queue 结果
- 已验证现有 task detail、owner acceptance、handoff receipt、alert triage 主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 第一次不再只在任务详情里表达 follow-up，而是开始回答“跨任务 backlog 现在长什么样”。这说明平台的高频协同能力开始从 task-scoped read model 走向 queue-scoped operational read。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道单条 follow-up 是否被接手，也能跨任务返回最小 owner queue。下一步最合理的动作，是继续把 queue 从 owner read 推进到 blocked / overdue / escalation projection，而不是急着引入过重的新表和新状态机。
