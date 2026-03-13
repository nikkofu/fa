# M1 Alert Triage Alert Cluster and Event Ingestion Direction Kickoff

## 日期

2026-03-12

## 同步目的

在 `告警分诊` workflow alignment checklist，以及 `follow-up / SLA`、`shift handoff receipt` 方向都已经建立后，继续把告警场景里最关键的专属对象单独收紧：`alert cluster / event-ingestion`。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert triage alert cluster and event ingestion direction`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把告警场景从“有对齐清单”推进到“有独立 cluster 与 ingestion 边界”

## 上一阶段完成基线

上一阶段已完成：

- `follow-up / SLA` read model and query direction note
- `班次交接` receipt / acknowledgement direction note
- alert triage 与通用 follow-up 的对象边界已进入明确设计问题

## 本阶段目标

1. 输出 `告警分诊` 的 alert cluster / event-ingestion direction note。
2. 明确 raw alert、cluster、triage draft 和 follow-up 的边界。
3. 说明 cluster 应先如何进入 task detail，再如何进入跨时间窗 query。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-alert-triage-alert-cluster-event-ingestion-direction.md`
- 项目状态同步

本阶段暂不交付：

- `alert_cluster` 领域模型代码
- event-ingestion API 代码
- cluster projection 实现
- `Scada / Andon / incident_log` connector 代码

## 风险与注意事项

- 不能把 raw alert 直接写成 triage task
- 不能把 alert cluster 写成 follow-up item
- 不能把 event-ingestion 包装成自动停线、自动消警或自动安全裁决

## 进入本阶段的理由

如果不把 cluster 和 ingestion 边界单独收紧，告警场景后续实现就很容易在四层对象之间来回混淆：原始告警、归并簇、分诊草稿和后续待办。只有把这层方向写清，告警场景才可能形成真实的事件到任务主链。

## 本阶段完成结果

- 已交付 alert triage alert-cluster / event-ingestion direction note
- 已明确 raw alert、cluster、triage draft、follow-up 四层对象边界
- 已把下一步从“高频场景方向”推进到“代码切口与 connector 基线”

## 实现摘要

这一阶段最重要的变化，是平台开始承认告警场景除了 evidence 和 follow-up 之外，还需要一层独立的 cluster / ingestion 对象。这个对象既不属于任务本体，也不等于后续待办，而是事件协同主线的中间层。

## 验证记录

已完成验证：

- 新方向文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道告警场景需要 connector、evidence 和 follow-up，也开始知道事件如何进入平台、如何形成 cluster、如何再进入 triage。下一步最合理的动作是为 `质量偏差隔离与处置建议` 补 mock `QMS` connector baseline，并开始评估最小 `follow_up_items / handoff_receipt / alert_cluster_drafts` 进入代码主链的切口。
