# M1 Shift Handoff Workflow Spec Kickoff

## 日期

2026-03-12

## 同步目的

在高频、高优先级 Agentic 功能优先级地图已经建立后，继续把排名第一的日常协同功能 `班次交接摘要与待办提取` 固化成正式 workflow specification baseline，避免“高频功能很重要”的判断继续停留在功能排序层。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`shift handoff workflow specification`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把高频功能排序真正落到第一条日常协同 workflow spec

## 上一阶段完成基线

上一阶段已完成：

- 高优先级 Agentic function priority map
- `Shift Handoff / Alert Triage / Follow-up Tracker / Repeated Anomaly Memory` 四类功能包排序
- 下一步文档重心切到 `班次交接` 和 `告警分诊`

## 本阶段目标

1. 为 `班次交接摘要与待办提取` 输出正式 workflow spec baseline。
2. 明确交接场景的角色、证据、待办、风险和升级边界。
3. 说明该场景与当前平台能力的复用点与缺口。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-shift-handoff-workflow-spec.md`
- 场景地图与高频功能地图中的文档链接
- 项目状态同步

本阶段暂不交付：

- shift log connector 实现
- follow-up / SLA 模型代码
- 交接摘要自动触发器
- 跨任务聚合视图

## 风险与注意事项

- 不能把低风险高频功能写成“只是摘要”，必须明确 follow-up 和升级边界
- 不能让交接摘要变成默认自动关单或自动重分配 owner 的入口
- 不能把当前没有的 `shift log / incident log` connector 写成已支持能力

## 进入本阶段的理由

如果高频功能只有排序没有 spec，后续产品和实现仍然会被更重的异常场景重新吸走注意力。先把 `班次交接摘要与待办提取` 规格化，才能真正把平台路线拉向“日常运营协同”。

## 本阶段完成结果

- 已交付 `班次交接摘要与待办提取` workflow spec baseline
- 已明确交接场景的摘要、follow-up、风险升级和禁止动作边界
- 已把高频功能探索从优先级地图推进到正式设计输入

## 实现摘要

这一阶段最重要的变化，是平台第一次为一个“高频、低风险、日常发生”的功能写出正式 workflow spec。它说明 FA 的扩展路线不再只由设备异常和质量偏差定义，而开始覆盖制造现场的日常协同节奏。

## 验证记录

已完成验证：

- 新 workflow spec 已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在不只是知道“哪些高频功能值得做”，而是已经冻结了第一条高频日常协同 spec。下一步最合理的动作是继续补 `产线告警聚合与异常分诊` spec，而不是重新回到更重但更低频的扩展方向。
