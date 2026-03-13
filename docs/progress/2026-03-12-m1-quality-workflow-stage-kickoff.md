# M1 Quality Deviation Workflow Spec Kickoff

## 日期

2026-03-12

## 同步目的

在制造行业场景全景已经建立后，继续把第二条最值得探索的候选场景 `质量偏差隔离与处置建议` 固化成正式 specification baseline，避免场景优先级已经明确，但质量场景仍停留在一行标题。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`quality deviation candidate workflow specification`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：开始本阶段前包含本地未提交的 `v0.2.0` 文档与验证演进；本阶段继续在其上推进

## 上一阶段完成基线

上一阶段已完成并推送：

- manufacturing scenario landscape
- 第二波场景优先级建议
- connector roadmap 的方向性排序

## 本阶段目标

1. 为 `质量偏差隔离与处置建议` 输出正式 spec baseline。
2. 明确质量场景的角色、证据、审批和回退边界。
3. 区分当前平台通用能力与质量场景尚待补齐能力。
4. 同步 README / roadmap / progress / journal / changelog。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-quality-deviation-workflow-spec.md`
- 场景全景文档中的质量场景链接
- 项目状态与下一步同步

本阶段暂不交付：

- QMS connector
- Quality Manager 审批策略实现
- 质量处置 draft API

## 风险与注意事项

- 不能把候选质量场景写成“当前已经打通”
- 必须明确质量放行、报废、冻结等动作仍不能自动执行
- 质量场景的治理边界应比设备异常场景更谨慎

## 进入本阶段的理由

如果第二波场景里最重要的质量场景迟迟没有规格文档，后续 connector、governance 和 approval 讨论就很容易继续围绕设备异常单点展开，难以证明平台对更高治理要求场景的准备程度。

## 本阶段完成结果

- 已交付 `质量偏差隔离与处置建议` 候选 workflow spec
- 已明确质量场景的目标输出、禁止动作和验收边界
- 已把质量场景从候选标题推进成正式设计输入

## 实现摘要

本阶段重点不是扩代码，而是扩“高价值但高治理复杂度场景”的设计清晰度。这样后续如果进入 `QMS / ERP / WMS` 连接器建设，就不再是盲目接系统，而是围绕一条明确业务路径展开。

## 验证记录

已完成验证：

- 新质量 workflow spec 已进入 `docs/design`
- 场景全景文档已链接质量 spec
- README / roadmap / changelog / progress / journal 已同步

## 阶段收口结论

FA 现在不仅知道“下一条值得做什么”，还知道“那条质量场景具体长什么样”。下一步最适合补的是质量场景的 connector / evidence / governance 对齐清单，而不是继续停留在概念层。
