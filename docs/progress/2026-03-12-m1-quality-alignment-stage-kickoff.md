# M1 Quality Workflow Alignment Kickoff

## 日期

2026-03-12

## 同步目的

在 `质量偏差隔离与处置建议` 已经形成候选 spec baseline 之后，继续把它推进到实现准备层，补齐与当前 `API / connector / evidence / governance / audit` 的对齐清单，避免质量场景继续停留在“规格已经有了，但落地入口仍然模糊”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`quality workflow alignment checklist`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在未提交的 `v0.2.0` 代码与文档演进上推进，本阶段重点是把质量场景从 spec 推到实现对齐层

## 上一阶段完成基线

上一阶段已完成：

- `质量偏差隔离与处置建议` 候选 workflow specification baseline
- 质量场景的角色、输出、禁止动作和回退边界冻结
- 第二波场景优先级与质量场景定位明确

## 本阶段目标

1. 输出质量场景与当前平台能力的正式对齐清单。
2. 明确 `QMS / MES / ERP / WMS / LIMS / SPC` 的 connector 优先级和 mock-first 路线。
3. 说明当前 `TaskEvidence`、governance 和 API 能复用什么，还缺什么。
4. 为后续 mock `QMS`、`Quality Manager` 审批策略和质量 draft output 建立顺序。
5. 同步 `README / roadmap / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-quality-workflow-alignment-checklist.md`
- 项目状态同步
- 阶段 kickoff 与 journal notes

本阶段暂不交付：

- mock `QMS` connector 实现
- `Quality Manager` 审批策略代码实现
- 质量 impact scope 的结构化 API 字段
- CAPA / disposition draft endpoint

## 风险与注意事项

- 不能把“quality target 已预埋”误写成“质量 connector 已可用”
- 不能把当前 `Safety Officer` 默认审批路径误包装成质量场景正式审批方案
- 不能为了写 checklist 而模糊自动放行、自动报废、自动冻结等禁止动作

## 进入本阶段的理由

如果 spec 已经有了，但没有对齐清单，后续实现就很容易重新掉回“想到哪个系统就接哪个系统”的模式。质量场景尤其不能这样推进，因为它的真正难点从来不是新增一个 connector trait，而是把证据、责任和审批角色对齐到位。

## 本阶段完成结果

- 已交付质量 workflow alignment checklist baseline
- 已明确 connector、evidence、governance、API 和 phased rollout 的主要差距
- 已把质量场景下一步实现顺序从“抽象讨论”推进成可执行建议

## 实现摘要

这一阶段最重要的判断是：质量场景当前最大的缺口不在 task lifecycle，也不在审批接口，而在三件基础能力上：

- 可读的 `QMS` 证据基线
- 正确的质量审批责任角色
- 能表达批次影响与处置建议的结构化输出

这让后续工作不再停留在“质量也值得做”，而是明确知道“先做什么、为什么先做它”。

## 验证记录

已完成验证：

- 新对齐清单已进入 `docs/design`
- `README / roadmap / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在对质量场景的认识，已经从“有一份 spec”推进到“有一张落地地图”。下一步最合理的实现动作是 mock `QMS` read-only baseline，而不是继续只做概念扩写。
