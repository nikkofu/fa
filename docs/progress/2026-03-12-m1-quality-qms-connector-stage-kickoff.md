# M1 Quality QMS Mock Connector Baseline Kickoff

## 日期

2026-03-12

## 同步目的

在 `质量偏差` workflow alignment checklist 已经建立后，继续把质量场景里最关键的运行缺口单独收紧：`mock QMS` connector baseline。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`quality qms mock connector baseline`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把质量场景从“可声明 Quality integration”推进到“能读到 QMS evidence”

## 上一阶段完成基线

上一阶段已完成：

- alert triage alert-cluster / event-ingestion direction note
- 高频场景的 receipt、cluster、follow-up 读层边界收紧
- 代码切口与 connector 基线已经成为下一层优先事项

## 本阶段目标

1. 输出 `质量偏差` 的 mock `QMS` connector baseline note。
2. 明确当前 `Quality / QMS` 在代码里预埋到了哪一层、缺口还在哪一层。
3. 定义第一版 `QMS` payload、registry、evidence、audit 对齐方式。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-quality-qms-mock-connector-baseline.md`
- 项目状态同步

本阶段暂不交付：

- `MockQualityConnector` 代码
- `Quality Manager` 审批策略代码
- lot / batch 结构化 query
- 真实 QMS 写入或 CAPA / NCR 创建

## 风险与注意事项

- 不能把 `IntegrationTarget::Quality` 误写成当前已打通 evidence 链路
- 不能把 `QualityContext` 的存在误写成 `QMS` connector 已存在
- 不能把 connector baseline 和质量审批策略问题混写成一起解决

## 进入本阶段的理由

如果不先补 `QMS` mock baseline，质量场景后续实现仍然会停留在“声明了 quality 集成目标，但 evidence 为空”的状态。只有把这层 baseline 写清，质量场景才可能进入真正可演示的证据主链。

## 本阶段完成结果

- 已交付 quality `QMS` mock connector baseline note
- 已明确 `Quality` target、`QualityContext`、registry、read plan 和 audit 的真实缺口
- 已把下一步从“connector 方向”推进到“最小读模型进入代码主链的切口”

## 实现摘要

这一阶段最重要的变化，是平台开始把质量场景的关键问题从“有没有 Quality 这个枚举”拉回到“默认运行链路到底能不能读到 QMS 证据”。这样质量路线才从名词预埋推进到真正可运行的证据基线。

## 验证记录

已完成验证：

- 新基线文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道质量场景需要 `QMS`，也开始知道 `QMS` 在默认运行主链里还缺什么。下一步最合理的动作是开始评估最小 `follow_up_items / handoff_receipt / alert_cluster_drafts` 进入 `tasks/{task_id}` 的代码切口。
