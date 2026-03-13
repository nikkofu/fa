# M1 Follow-up and SLA Read Model Direction Kickoff

## 日期

2026-03-12

## 同步目的

在 `follow-up / SLA` 通用模型对齐清单，以及 `班次交接`、`告警分诊` 的实现对齐清单都已经建立后，继续把这层通用协同对象推进到 read model 和 query 方向，避免对象模型继续停留在“知道缺什么”但还不知道“应该怎么读、怎么查”。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up and SLA read model and query direction`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 follow-up / SLA 从对象缺口说明推进到任务级读模型和跨任务 query 方向

## 上一阶段完成基线

上一阶段已完成：

- `follow-up / SLA` 通用模型对齐清单
- `班次交接` workflow alignment checklist
- `告警分诊` workflow alignment checklist

## 本阶段目标

1. 输出 `follow-up / SLA` 的 read model 和 query direction note。
2. 明确哪些对象应先进入 `tasks/{task_id}`，哪些必须走跨任务聚合查询。
3. 说明当前 task repository、audit query 和 API 基线对这层读模型的支持边界。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-follow-up-sla-read-model-query-direction.md`
- 项目状态同步

本阶段暂不交付：

- `follow_up_items` 领域模型代码
- follow-up 聚合查询 API 代码
- SLA 评估器代码
- receipt / alert cluster 场景专属对象代码

## 风险与注意事项

- 不能把跨任务查询继续建立在 task JSON blob 扫描上
- 不能把 audit replay 误当成 backlog query
- 不能把 `班次交接` receipt 或 `告警分诊` alert cluster 直接吞进通用 follow-up 状态机

## 进入本阶段的理由

如果只停留在对齐清单，平台仍然只知道“缺 follow-up / SLA 对象”，却还不知道“这些对象应该先挂在哪、怎样被查询、哪些必须单独做 projection”。只有把 read model 和 query 方向写清，后续实现才不会反复摇摆。

## 本阶段完成结果

- 已交付 follow-up / SLA read model and query direction note
- 已明确 task-scoped read model、cross-task queue query 和 SLA monitoring 的分层方向
- 已把下一步从“通用对象方向”推进到“场景专属读模型边界”

## 实现摘要

这一阶段最重要的变化，是平台开始把 follow-up / SLA 当成“读层设计问题”来处理，而不只是继续抽象对象名词。这会直接决定后续 API、projection、审计和场景对象怎样拼成一条稳定主线。

## 验证记录

已完成验证：

- 新方向文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道 follow-up / SLA 缺什么对象，也开始知道这些对象应该怎样进入任务详情和跨任务查询。下一步最合理的动作是继续收敛 `班次交接` 的 receipt / acknowledgement 方向，以及 `告警分诊` 的 alert cluster / event-ingestion 方向。
