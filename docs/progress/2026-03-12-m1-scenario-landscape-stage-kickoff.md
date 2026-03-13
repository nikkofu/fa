# M1 Manufacturing Scenario Landscape Kickoff

## 日期

2026-03-12

## 同步目的

在首条 pilot workflow 已经冻结、治理和验证基线逐步成形之后，继续把对制造行业 Agentic AI 应用场景的理解从“少数候选 workflow”推进为一份更完整的场景全景，用于指导后续版本路线、connector 优先级和第二波场景选择。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`manufacturing scenario landscape`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：开始本阶段前包含本地未提交的 `v0.2.0` 持续演进变更；本阶段在其上继续推进

## 上一阶段完成基线

上一阶段已完成并推送：

- approval role enforcement
- sandbox-safe smoke baseline
- README / roadmap / QA 文档同步

## 本阶段目标

1. 输出制造行业 Agentic AI 场景全景文档。
2. 对不同场景建立价值、风险、系统依赖和治理模式分层。
3. 给出当前 pilot 之后的第二波场景推进建议。
4. 把场景探索结果同步进 roadmap、README 和 planning 入口。

## 本阶段交付边界

本阶段计划交付：

- `docs/planning/manufacturing-agentic-ai-scenario-landscape.md`
- 场景分层与优先级建议
- 当前 pilot 后续扩展顺序建议
- progress / journal / changelog / roadmap / README 同步

本阶段暂不交付：

- 第二条 workflow specification 正式冻结
- 新 connector 实现
- 新 API 或新领域模型

## 风险与注意事项

- 场景探索不能回到“什么都能做”的平台叙事
- 必须明确哪些场景适合当前阶段，哪些必须后做
- 场景排序要兼顾业务价值、组织接受度和治理复杂度

## 进入本阶段的理由

如果只有一条 pilot workflow，平台仍然容易被误解成“只服务单个设备异常用例”。补齐场景全景，才能更清楚地回答：这个平台到底服务哪些制造问题、接下来该先扩到哪里、为什么不是别的方向。

## 本阶段完成结果

- 已交付制造行业 Agentic AI 场景全景文档
- 已形成按业务域和治理层级划分的场景地图
- 已形成当前 pilot 后第二波场景推进建议
- planning / roadmap / README / changelog 已同步

## 实现摘要

本阶段没有继续扩功能，而是先扩“问题地图”。这样做可以让后续 connector、evidence 和 governance 的投资不再依赖临时灵感，而是更明确地贴着制造场景组合推进。

## 验证记录

已完成验证：

- 文档结构已进入 `planning/` 正式入口
- roadmap 已链接新场景文档
- README 已同步当前能力与下一步优先级

## 阶段收口结论

FA 现在不再只有“首条 pilot workflow”的单点视角，而开始拥有“制造行业应用场景组合”的视角。下一步更适合基于这份场景地图，冻结第二条候选 workflow 的具体规格。
