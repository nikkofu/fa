# M1 Follow-up Task Read Model Implementation Cut Kickoff

## 日期

2026-03-12

## 同步目的

在 `follow-up / SLA` read model direction note 已经建立后，继续把它推进到真正可落代码的最小实现切口，避免后续一动手又退回“概念很多、插点不清”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up task read model implementation cut`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `follow_up_items` 进入 `tasks/{task_id}` 的最小代码切口收紧到真实文件和真实边界

## 上一阶段完成基线

上一阶段已完成：

- quality `QMS` mock connector baseline note
- 高频读层对象与 connector 基线的优先级已经收敛到代码切口层
- 下一层最先值得推进的是 `follow_up_items` task detail schema cut

## 本阶段目标

1. 输出 `follow_up_items` 进入任务详情的 implementation cut note。
2. 明确 `TrackedTaskState`、`tasks/{task_id}`、repository 和 smoke 的真实落点。
3. 说明 Phase A 先做什么、暂时不做什么，避免把任务详情 cut 膨胀成新 backlog 系统。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-follow-up-task-read-model-implementation-cut.md`
- 项目状态同步

本阶段暂不交付：

- `follow_up_items` 代码实现
- 跨任务 follow-up queue API
- follow-up item 写接口
- item 级 audit projection

## 风险与注意事项

- 不能把 `PlannedStep.owner` 误当成正式 follow-up owner
- 不能把 task detail schema cut 和跨任务 backlog query 混成一件事
- 不能把 `handoff_receipt`、`alert_cluster_drafts`、`QMS` code-cut 一起塞进同一批改动

## 进入本阶段的理由

如果只停留在 direction note，团队仍然知道“`follow_up_items` 应该进任务详情”，但还不知道“第一刀到底改哪几个 struct、哪几个接口、哪几类测试”。只有把 implementation cut 写清，后续代码才能最小、稳定地进入主链。

## 本阶段完成结果

- 已交付 follow-up task read model implementation cut note
- 已明确 `TrackedTaskState` 是第一优先级插点，repository 与 API 可原样复用
- 已把下一步从“抽象 read model 方向”推进到“交接 receipt / 告警 cluster 的对应代码切口”

## 实现摘要

这一阶段最重要的变化，是平台开始把 `follow-up / SLA` 从“应该有什么对象”进一步推进到“第一刀该落在哪个代码锚点”。这会直接决定后续 task detail、smoke、持久化和高频场景对象如何收敛成同一条可实施主线。

## 验证记录

已完成验证：

- 新 implementation cut 文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道 `follow_up_items` 应该进入任务详情，也知道第一刀最合理的代码切口是什么。下一步最合理的动作，是继续为 `handoff_receipt` 和 `alert_cluster_drafts` 输出同类型 implementation cut note，然后再进入最小代码实现。
