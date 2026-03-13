# M1 Alert Triage Task Read Model Cut Kickoff

## 日期

2026-03-12

## 同步目的

在 `alert cluster / event-ingestion` direction note 已经建立后，继续把它推进到真正可落代码的最小实现切口，避免后续实现又把 raw alert、cluster、triage draft 和 follow-up 混成一个对象。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert triage task read model implementation cut`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `alert_cluster_drafts` 进入 `tasks/{task_id}` 的最小代码切口收紧到真实文件与真实兼容边界

## 上一阶段完成基线

上一阶段已完成：

- shift handoff receipt task read model implementation cut note
- 高频 task-detail read-model code cut 方法已经在 `follow_up_items` 与 `handoff_receipt` 上收敛
- 下一层最值得推进的是告警 cluster draft 的对应代码切口

## 本阶段目标

1. 输出 `alert_cluster_drafts` 进入告警任务详情的 implementation cut note。
2. 明确 `TrackedTaskState`、`tasks/{task_id}`、repository 和 smoke 的真实落点。
3. 明确 cluster draft 与 raw alert、follow-up、ingestion 的边界，避免事件对象和执行对象混写。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-alert-triage-task-read-model-implementation-cut.md`
- 项目状态同步

本阶段暂不交付：

- `alert_cluster_drafts` 代码实现
- event-ingestion API
- cluster projection / query
- `Scada / Andon / incident_log` connector 代码

## 风险与注意事项

- 不能把 raw alert 直接写进 task detail 顶层
- 不能把 cluster draft 写成 follow-up item
- 不能因为 ingestion 代码还没落地，就让 cluster schema cut 被迫停住

## 进入本阶段的理由

如果只停留在 direction note，团队仍然知道“告警场景需要 cluster draft”，但还不知道“第一刀应该扩哪两个字段、旧任务 JSON 会不会坏、是否必须先做 ingestion”。只有把 implementation cut 写清，告警场景后续代码才不会重新发散。

## 本阶段完成结果

- 已交付 alert triage task read model implementation cut note
- 已明确 `TrackedTaskState` 是 cluster draft 第一优先级插点，repository 与 API 可以继续复用
- 已把下一步从“高频 task-detail 对象继续评估”推进到“三类 schema cut 开始进入代码实现”

## 实现摘要

这一阶段最重要的变化，是平台开始把 `alert_cluster_drafts` 从“需要独立对象”进一步推进到“最小代码切口应该落在哪”。这会直接决定告警 task detail、持久化、smoke 与后续 ingestion / query 如何沿同一条主线推进。

## 验证记录

已完成验证：

- 新 implementation cut 文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道告警场景需要 `alert_cluster_drafts`，也知道第一刀最合理的代码切口是什么。下一步最合理的动作，是把 `follow_up_items`、`handoff_receipt` 和 `alert_cluster_drafts` 的最小 schema cut 依次推进到代码实现，并评估 mock `QMS` baseline 的代码切口。
