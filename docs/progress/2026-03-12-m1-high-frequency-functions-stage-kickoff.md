# M1 High-frequency Agentic Functions Kickoff

## 日期

2026-03-12

## 同步目的

在制造场景全景、质量候选 workflow spec 和质量对齐清单都已经建立后，继续把探索从“workflow 候选”推进到“高频、高优先级功能优先级”。目标是明确哪些 Agentic 功能最值得先产品化，而不是继续只围绕少数重场景讨论。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`high-frequency Agentic function priority map`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 的本地未提交演进上推进，本阶段重点是识别更高频、更高优先级且更适合先产品化的功能包

## 上一阶段完成基线

上一阶段已完成：

- 质量候选 workflow 的 connector / evidence / governance / API 对齐清单
- 第二条质量候选 workflow 的实现顺序建议
- 当前平台在质量场景中的主要缺口识别

## 本阶段目标

1. 输出制造现场高频、高优先级 Agentic 功能优先级地图。
2. 区分哪些应先做成平台通用 function pack，哪些再进入 workflow spec。
3. 给出最值得优先产品化的 3 到 4 个功能包。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/planning/high-frequency-agentic-function-priority-map.md`
- 项目状态同步
- 阶段 kickoff 与 journal notes

本阶段暂不交付：

- 新 connector 实现
- 事件驱动任务生成代码实现
- follow-up / SLA 领域模型代码
- 新 workflow API

## 风险与注意事项

- 不能把“高频”误写成“低价值”
- 不能只按场景标题排序，而要识别跨场景复用的 function pack
- 不能让平台路线继续只偏向低频的高风险重场景

## 进入本阶段的理由

如果平台只围绕设备异常和质量偏差这些重要但相对更重的场景扩展，就容易失去进入日常经营节奏的机会。高频功能才更容易建立组织使用习惯，也是把平台做成“协同操作层”而不只是“异常处理器”的关键。

## 本阶段完成结果

- 已交付高频、高优先级 Agentic 功能优先级地图
- 已明确 `Shift Handoff / Alert Triage / Follow-up Tracker / Repeated Anomaly Memory` 是最值得先产品化的功能包
- 已把下一步文档重心切到 `班次交接` 和 `告警分诊` 这两条更高频的 baseline spec

## 实现摘要

这一阶段最重要的变化，不是再新增一个场景名字，而是把产品视角从“做哪条 workflow”进一步推进到“哪些能力最值得先做成平台功能”。

这意味着后续路线将不再只围绕高风险治理，而会更强调：

- 高频使用
- 日常协同
- 跨场景复用
- 快速形成组织习惯

## 验证记录

已完成验证：

- 新优先级地图已进入 `docs/planning`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 下一步更值得优先推进的，不只是更“重”的 workflow，而是更“常用”的 Agentic function pack。最合理的后续动作是继续冻结 `班次交接摘要与待办提取` 和 `产线告警聚合与异常分诊` 的 baseline spec。
