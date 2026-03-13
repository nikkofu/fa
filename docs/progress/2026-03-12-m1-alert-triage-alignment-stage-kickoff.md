# M1 Alert Triage Workflow Alignment Kickoff

## 日期

2026-03-12

## 同步目的

在 `产线告警聚合与异常分诊` workflow spec 和 `follow-up / SLA`、`shift handoff` 对齐清单都已经建立后，继续把告警场景推进到实现准备层，补齐 connector / evidence / governance / event-ingestion 的正式对齐清单。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert triage workflow alignment checklist`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把高频事件协同场景从 spec 推到 connector / evidence / event-ingestion 对齐层

## 上一阶段完成基线

上一阶段已完成：

- `班次交接摘要与待办提取` workflow alignment checklist
- 高频协同 connector / evidence / follow-up / receipt 缺口识别方法基线
- 告警场景依赖的 follow-up / SLA 通用模型缺口冻结

## 本阶段目标

1. 输出 `产线告警聚合与异常分诊` 的 connector / evidence / governance / event-ingestion 对齐清单。
2. 明确 `SCADA / Andon / MES / incident log / CMMS` 的接入优先级与 mock-first 路线。
3. 说明告警场景在 evidence、governance、follow-up、API 上与当前平台的差距。
4. 同步 `README / roadmap / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-alert-triage-alignment-checklist.md`
- 项目状态同步

本阶段暂不交付：

- `SCADA / Andon / incident log` connector 代码实现
- 事件流 ingestion endpoint
- alert cluster / triage read model 代码
- 跨时间窗告警聚合视图

## 风险与注意事项

- 不能把 `Scada` target 写成当前已经可读的默认 connector
- 不能把 `Andon / incident_log` 写成当前已存在的一等 integration target
- 不能把 triage confirmation 误写成 formal approval
- 不能把告警分诊包装成自动停线、自动消警或自动安全裁决

## 进入本阶段的理由

如果告警场景只有 spec 没有对齐清单，后续实现很容易重新滑回“先做个告警总结 demo”的方向。只有把 connector、evidence、governance 和 event-ingestion 缺口写清，告警场景才可能进入真正可实施状态。

## 本阶段完成结果

- 已交付 alert triage workflow alignment checklist baseline
- 已明确告警场景的 connector、evidence、governance、API 和 event-ingestion 主要差距
- 已把告警场景从 workflow spec 推进到更接近实现的对齐层

## 实现摘要

这一阶段最重要的变化，是平台不再只知道“告警分诊值得做”，而开始知道“告警流怎样在不越过治理边界的前提下进入任务主链、证据层和后续协同层”。

## 验证记录

已完成验证：

- 新对齐清单已进入 `docs/design`
- `README / roadmap / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不只是知道告警场景需要 spec，也开始知道它需要哪些 connector、evidence 和 event-ingestion 层。下一步最合理的动作是开始收敛 follow-up / SLA read model 与 query 方向，并继续收紧 shift handoff 的 receipt / acknowledgement 语义。
