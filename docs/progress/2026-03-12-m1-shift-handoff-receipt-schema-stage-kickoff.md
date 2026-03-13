# M1 Shift Handoff Receipt Task Detail Schema Cut Kickoff

## 日期

2026-03-12

## 同步目的

在 `handoff_receipt` implementation cut note 已经建立后，继续把它推进到真正进入代码主链的最小 schema cut，避免交接任务详情一直停留在“知道应该有 receipt”，但 API 里还没有正式字段的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`shift handoff receipt task detail schema cut`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `handoff_receipt / handoff_receipt_summary` 以兼容旧任务 JSON 的方式真实接入 `TrackedTaskState`

## 上一阶段完成基线

上一阶段已完成：

- follow-up task read model schema cut
- `tasks/intake` 与 `tasks/{task_id}` 已具备空默认 `follow_up_items / follow_up_summary`
- 下一层最值得推进的是交接 receipt 的对应 task detail schema cut

## 本阶段目标

1. 把 `handoff_receipt / handoff_receipt_summary` 加入 `TrackedTaskState`。
2. 保持 `tasks/{task_id}`、file repository、SQLite repository 和旧 JSON 的兼容。
3. 补齐 repository round-trip、API contract 和 sandbox smoke 断言。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `TrackedTaskState.handoff_receipt`
- `TrackedTaskState.handoff_receipt_summary`
- compatibility / repository / API contract 测试与 smoke 断言
- 项目状态同步

本阶段暂不交付：

- 非空 `handoff_receipt` 自动生成逻辑
- acknowledgement action API
- 跨班次 receipt queue API
- receipt 专属审计事件

## 风险与注意事项

- 不能破坏旧任务 JSON 的反序列化
- 不能为了 schema cut 去改动 approval、task lifecycle 或 route 结构
- 不能把默认空 schema 误写成“交接闭环已经完整实现”

## 进入本阶段的理由

如果 `handoff_receipt` 一直不进入代码，交接场景仍然只能靠 follow-up 和 evidence 间接表达，平台无法正式回答“交接包有没有被接住”。只有先把 receipt 接进任务详情，后续 acknowledgement 和 queue 才有稳定落点。

## 本阶段完成结果

- 已交付最小 `handoff_receipt / handoff_receipt_summary` task detail schema cut
- 已验证旧 JSON 兼容、repository round-trip、API contract 与 sandbox smoke
- 已把下一步从“继续写 receipt 切口说明”推进到“alert cluster schema cut 和非空 draft 填充”

## 实现摘要

这一阶段最重要的变化，是 `TrackedTaskState` 第一次正式拥有了交接 receipt task detail 字段，而且这些字段不会破坏现有 file / SQLite 持久化和旧任务回读。这使得交接任务开始具备“交接包是否被接住”的正式 API contract，而不是只能靠口头语义描述。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅知道交接场景需要 `handoff_receipt`，也已经把最小 schema cut 真实接进了 API 和持久化主链。下一步最合理的动作，是按同样方法继续把 `alert_cluster_drafts` 推进到代码实现，并开始为交接场景填充非空 receipt draft。
