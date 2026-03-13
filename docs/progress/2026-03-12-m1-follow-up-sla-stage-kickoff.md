# M1 Follow-up and SLA Alignment Kickoff

## 日期

2026-03-12

## 同步目的

在 `班次交接摘要与待办提取` 与 `产线告警聚合与异常分诊` 两条高频功能 spec 都已经建立后，继续把它们共同暴露出的模型缺口推进成正式对齐清单：`follow-up / owner / due date / SLA`。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up and SLA model alignment checklist`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把高频协同里最核心的通用对象模型缺口冻结下来

## 上一阶段完成基线

上一阶段已完成：

- `产线告警聚合与异常分诊` workflow spec baseline
- 高频事件输入场景的分级、路由和升级边界冻结
- 高时效协同场景正式进入设计输入

## 本阶段目标

1. 输出 `follow-up / owner / due date / SLA` 的通用模型对齐清单。
2. 明确这些对象与当前 `TaskRequest / TaskRecord / PlannedStep / TaskEvidence / Audit` 的差距。
3. 说明这些对象如何支撑 `班次交接`、`告警分诊` 和后续质量场景。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-follow-up-sla-model-alignment-checklist.md`
- 高频功能地图中的链接与下一步状态同步
- 项目状态同步

本阶段暂不交付：

- follow-up 领域模型代码
- SLA 规则引擎代码
- follow-up 查询 API
- 跨任务聚合视图实现

## 风险与注意事项

- 不能把 `PlannedStep.owner` 误当成 follow-up owner
- 不能在模型未冻结前就急着定义大量接口
- 不能把“提醒谁跟进”写成“系统替业务分配责任”

## 进入本阶段的理由

如果没有这层通用模型，`班次交接` 和 `告警分诊` 仍然只能停留在“能总结、能建议”，但还不能稳定闭环。先把 follow-up / SLA 模型收紧，后续高频功能才不会各自发散。

## 本阶段完成结果

- 已交付 follow-up / SLA 通用模型对齐清单
- 已明确当前任务模型、evidence、governance 和 audit 的主要缺口
- 已把下一步从“继续写场景 spec”推进到“开始收敛通用 read model 与 query 方向”

## 实现摘要

这一阶段最重要的变化，是平台开始把“协同对象”单独看待，而不是继续把所有内容塞回任务描述文本。这样后续平台建设才会更像通用操作层，而不是一组互相独立的 workflow 文档。

## 验证记录

已完成验证：

- 新对齐清单已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅有高频功能 spec，也开始补这些功能共同依赖的通用模型层。下一步最合理的动作是分别为 `班次交接` 与 `告警分诊` 输出 connector / evidence 对齐清单，并开始收敛 read model 和 query 方向。
