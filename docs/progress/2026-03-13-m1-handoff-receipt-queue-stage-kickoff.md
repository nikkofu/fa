# M1 Handoff Receipt Queue Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `shift handoff` 已拥有 task-scoped `handoff_receipt`、显式 acknowledgement 和 escalation action 后，继续把它推进到跨班次 receipt queue read。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`handoff receipt queue`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `shift handoff` receipt 从 task-scoped action 推进到最小 cross-shift queue read

## 上一阶段完成基线

上一阶段已完成：

- `shift handoff` receipt 已能在 task detail 中返回
- `handoff-receipt/acknowledge` 与 `handoff-receipt/escalate` 已能表达接收与升级语义
- 下一层最值得推进的是 cross-shift receipt queue，而不是立即引入更重的 projection 表

## 本阶段目标

1. 增加最小 `GET /api/v1/handoff-receipts` cross-shift queue read。
2. 继续复用 repository 中已持久化的 `TrackedTaskState` 扫描，不引入新的 receipt projection。
3. 支持 `task_id / shift_id / receipt_status / receiving_role / receiving_actor_id / overdue_only / has_exceptions / escalated_only` 过滤。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `HandoffReceiptQueueQuery / HandoffReceiptQueueItemView`
- `WorkOrchestrator::list_handoff_receipts(...)`
- `GET /api/v1/handoff-receipts`
- cross-shift queue 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- dedicated receipt projection 表
- 独立 receipt aging monitor
- `expired` 的写入状态机
- receipt completion 或 receipt-level reassignment action

## 风险与注意事项

- 不能把 receipt queue 混成 follow-up owner queue
- 不能把 `expired` 误当成已持久化状态；第一版只在 query 时计算
- 不能把 cross-shift queue 和正式治理升级动作混成一条写路径

## 进入本阶段的理由

如果系统只能在单个任务里回答“这份交接包现在是什么状态”，但不能跨任务回答“哪些交接包超时未接、哪些带异议、哪些已升级”，那它仍然不够贴近交接班和班组值班场景。先补最小 cross-shift queue read，能最快验证 receipt 主链是否足以承接这一层运营视图。

## 本阶段完成结果

- 已为 `shift handoff` receipt 增加最小 cross-shift queue read API
- 已验证 queue 会聚合多条 `handoff_receipt`
- 已验证 `overdue_only / has_exceptions / escalated_only / shift_id / receiving_actor_id` 过滤成立
- 已验证 file-backed sandbox 模式重启后仍可回读 receipt queue 结果
- 已验证现有 task detail、follow-up queue、acknowledgement 和 escalation 主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 不再只在单任务里表达交接回执，而是开始回答“跨班次交接 backlog 现在长什么样”。这说明平台的交接协同能力开始从 task-scoped receipt 走向 cross-shift operational queue。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能在任务详情里返回 `handoff_receipt`，也能跨班次返回最小 receipt queue。下一步最合理的动作，是评估何时把 repository-scan queue 推进到 dedicated projection / receipt aging monitor。
