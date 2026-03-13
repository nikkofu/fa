# M1 Alert Triage Workflow Spec Kickoff

## 日期

2026-03-12

## 同步目的

在 `班次交接摘要与待办提取` 已经完成规格化之后，继续把排名第二的高频功能 `产线告警聚合与异常分诊` 固化成正式 workflow specification baseline，避免“事件驱动协同”继续只停留在功能描述层。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert triage workflow specification`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把高频事件协同功能冻结成正式 spec

## 上一阶段完成基线

上一阶段已完成：

- `班次交接摘要与待办提取` workflow spec baseline
- 高频日常协同场景的摘要、follow-up 和升级边界冻结
- 高频功能主线已从功能排序推进到正式设计输入

## 本阶段目标

1. 为 `产线告警聚合与异常分诊` 输出正式 workflow spec baseline。
2. 明确告警场景的聚合、去重、分级、路由和升级边界。
3. 说明该场景与当前平台能力的复用点与缺口。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-alert-triage-workflow-spec.md`
- 场景地图与高频功能地图中的文档链接
- 项目状态同步

本阶段暂不交付：

- `SCADA / Andon` connector 实现
- 事件驱动任务入口代码
- 告警簇 / 去重策略对象模型代码
- 告警聚合可视化视图

## 风险与注意事项

- 不能把告警分诊写成默认自动停线或自动消警入口
- 不能把当前没有的 `SCADA / Andon / incident log` connector 写成已支持能力
- 不能把安全、质量和停线风险的裁决权交给模型

## 进入本阶段的理由

如果高频功能只覆盖人工交接，而不进入告警流，平台仍然离制造现场最密集的实时协同输入很远。先把 `产线告警聚合与异常分诊` 规格化，才能让 FA 真正向事件驱动协同迈进一步。

## 本阶段完成结果

- 已交付 `产线告警聚合与异常分诊` workflow spec baseline
- 已明确告警场景的去重、分级、路由、升级和禁止动作边界
- 已把高频功能主线进一步推进到事件驱动协同方向

## 实现摘要

这一阶段最重要的变化，是平台不再只定义“日常交接”这类低频率窗口动作，而开始正面定义“实时告警流”这种更高时效输入。这样后续路线才可能自然进入事件到任务的受控转换。

## 验证记录

已完成验证：

- 新 workflow spec 已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经拥有两条高频功能 spec：一条面向交接，一条面向告警。下一步最合理的动作是补 `follow-up owner / SLA` 通用模型对齐清单，而不是继续堆更多未对齐的场景标题。
