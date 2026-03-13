# M1 Alert Triage Task Detail Schema Cut Kickoff

## 日期

2026-03-12

## 同步目的

在 `alert_cluster_drafts` implementation cut note 已经建立后，继续把它推进到真正进入代码主链的最小 schema cut，避免告警任务详情一直停留在“知道应该有 cluster draft”，但 API 里还没有正式字段的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert triage task detail schema cut`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `alert_cluster_drafts / alert_triage_summary` 以兼容旧任务 JSON 的方式真实接入 `TrackedTaskState`

## 上一阶段完成基线

上一阶段已完成：

- shift handoff receipt task detail schema cut
- `tasks/intake` 与 `tasks/{task_id}` 已具备空默认 `handoff_receipt / handoff_receipt_summary`
- 下一层最值得推进的是告警 cluster 的对应 task detail schema cut

## 本阶段目标

1. 把 `alert_cluster_drafts / alert_triage_summary` 加入 `TrackedTaskState`。
2. 保持 `tasks/{task_id}`、file repository、SQLite repository 和旧 JSON 的兼容。
3. 补齐 repository round-trip、API contract 和 sandbox smoke 断言。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `TrackedTaskState.alert_cluster_drafts`
- `TrackedTaskState.alert_triage_summary`
- compatibility / repository / API contract 测试与 smoke 断言
- 项目状态同步

本阶段暂不交付：

- 非空 `alert_cluster_drafts` 自动生成逻辑
- event-ingestion API
- cluster query / projection
- cluster 专属审计事件

## 风险与注意事项

- 不能破坏旧任务 JSON 的反序列化
- 不能为了 schema cut 去改动 task lifecycle、ingestion 边界或 route 结构
- 不能把默认空 schema 误写成“告警聚类与分诊已经完整实现”

## 进入本阶段的理由

如果 `alert_cluster_drafts` 一直不进入代码，告警场景仍然只能靠 evidence 和 follow-up 间接表达，平台无法正式回答“哪些信号已经形成需要分诊的异常簇”。只有先把 cluster draft 接进任务详情，后续 ingestion 和 cluster query 才有稳定落点。

## 本阶段完成结果

- 已交付最小 `alert_cluster_drafts / alert_triage_summary` task detail schema cut
- 已验证旧 JSON 兼容、repository round-trip、API contract 与 sandbox smoke
- 已把下一步从“继续写 cluster 切口说明”推进到“非空 draft 填充和 QMS 代码切口”

## 实现摘要

这一阶段最重要的变化，是 `TrackedTaskState` 第一次正式拥有了告警 cluster task detail 字段，而且这些字段不会破坏现有 file / SQLite 持久化和旧任务回读。这使得告警任务开始具备“哪些信号已形成异常簇”的正式 API contract，而不是只能靠 evidence 文本表达。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅知道告警场景需要 `alert_cluster_drafts`，也已经把最小 schema cut 真实接进了 API 和持久化主链。下一步最合理的动作，是开始为高频场景填充非空 `follow_up_items / handoff_receipt / alert_cluster_drafts` draft，并评估 mock `QMS` baseline 的代码切口。
