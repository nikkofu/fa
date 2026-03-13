# M1 Shift Handoff Receipt Task Read Model Cut Kickoff

## 日期

2026-03-12

## 同步目的

在 `shift handoff receipt / acknowledgement` direction note 已经建立后，继续把它推进到真正可落代码的最小实现切口，避免后续实现又把 receipt、follow-up accepted owner 和 approval 混在一起。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`shift handoff receipt task read model implementation cut`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `handoff_receipt` 进入 `tasks/{task_id}` 的最小代码切口收紧到真实文件与真实兼容边界

## 上一阶段完成基线

上一阶段已完成：

- follow-up task read model implementation cut note
- task detail 层最小 read-model code cut 方法已经在 `follow_up_items` 上收敛
- 下一层最值得推进的是交接 receipt 的对应代码切口

## 本阶段目标

1. 输出 `handoff_receipt` 进入交接任务详情的 implementation cut note。
2. 明确 `TrackedTaskState`、`tasks/{task_id}`、repository 和 smoke 的真实落点。
3. 明确 receipt 与 `follow_up_items` 的依赖关系，避免两个高频对象互相阻塞。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-shift-handoff-task-receipt-read-model-implementation-cut.md`
- 项目状态同步

本阶段暂不交付：

- `handoff_receipt` 代码实现
- acknowledgement action API
- 跨班次 receipt queue
- `shift_log / incident_log` connector 代码

## 风险与注意事项

- 不能把 receipt 状态接进 approval
- 不能把 package-level receipt 写成 item-level owner acceptance
- 不能因为 `follow_up_items` 代码未落地，就让 receipt schema cut 被迫停住

## 进入本阶段的理由

如果只停留在 direction note，团队仍然知道“交接场景需要独立 receipt”，但还不知道“第一刀应该扩哪两个字段、要不要加新接口、旧任务 JSON 会不会坏”。只有把 implementation cut 写清，交接场景后续代码才不会重新发散。

## 本阶段完成结果

- 已交付 shift handoff receipt task read model implementation cut note
- 已明确 `TrackedTaskState` 是 receipt 第一优先级插点，repository 与 API 可以继续复用
- 已把下一步从“交接 read-model 方向”推进到“alert cluster 对应 implementation cut”和“follow-up code cut”

## 实现摘要

这一阶段最重要的变化，是平台开始把 `handoff_receipt` 从“需要独立对象”进一步推进到“最小代码切口应该落在哪”。这会直接决定交接 task detail、持久化、smoke 与后续 acknowledgement action 如何沿同一条主线推进。

## 验证记录

已完成验证：

- 新 implementation cut 文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道交接场景需要 `handoff_receipt`，也知道第一刀最合理的代码切口是什么。下一步最合理的动作，是继续为 `alert_cluster_drafts` 输出同类型 implementation cut note，然后再把 `follow_up_items` 与 `handoff_receipt` 的最小 schema cut 推进到代码实现。
