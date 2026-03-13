# M1 Shift Handoff Workflow Alignment Kickoff

## 日期

2026-03-12

## 同步目的

在 `班次交接摘要与待办提取` workflow spec 和 `follow-up / SLA` 通用模型对齐清单都已经建立后，继续把交接场景推进到实现准备层，补齐 connector / evidence / SLA 的正式对齐清单。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`shift handoff workflow alignment checklist`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把高频交接场景从 spec 推到 connector / evidence / receipt / SLA 对齐层

## 上一阶段完成基线

上一阶段已完成：

- follow-up / owner / due date / SLA 通用模型对齐清单
- 高频协同对象与当前任务模型的主要缺口识别
- 后续高频功能的 read model 和 query 方向初步收紧

## 本阶段目标

1. 输出 `班次交接摘要与待办提取` 的 connector / evidence / SLA 对齐清单。
2. 明确 `MES / shift log / incident log / task history / CMMS` 的接入优先级与 mock-first 路线。
3. 说明交接场景在 evidence、follow-up、receipt、SLA 上与当前平台的差距。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-shift-handoff-alignment-checklist.md`
- 高频功能地图中的文档链接
- 项目状态同步

本阶段暂不交付：

- shift log / incident log connector 代码实现
- 交接 receipt / acknowledgement 领域模型代码
- 跨班次 unresolved 视图
- 交接超时规则引擎

## 风险与注意事项

- 不能把 `shift log / incident log` 写成当前已可用 connector
- 不能把交接 receipt 误写成正式审批
- 不能把交接摘要的文本输出误包装成结构化遗留事项能力已经具备

## 进入本阶段的理由

如果交接场景只有 spec 没有对齐清单，后续实现就会再次滑回“先做个摘要 demo”的方向。只有把 connector、evidence、follow-up 和 receipt 缺口写清，交接场景才可能进入真正可实施状态。

## 本阶段完成结果

- 已交付 shift handoff workflow alignment checklist baseline
- 已明确交接场景的 connector、evidence、follow-up、receipt 和 SLA 主要差距
- 已把下一步从“交接 workflow spec”推进到更接近实现的对齐层

## 实现摘要

这一阶段最重要的变化，是平台不再把交接场景当作一个轻量摘要功能，而是开始把它当作“高频输入 + 遗留事项 + 接收确认 + 时限升级”的正式协同对象来设计。

## 验证记录

已完成验证：

- 新对齐清单已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不只是知道交接场景值得做，也开始知道它要怎么做。下一步最合理的动作是为 `产线告警聚合与异常分诊` 输出同层级的 connector / evidence / event-ingestion 对齐清单。
