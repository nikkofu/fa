# M1 Shift Handoff Receipt Direction Kickoff

## 日期

2026-03-12

## 同步目的

在 `班次交接` workflow alignment checklist 和 `follow-up / SLA` read model direction 都已经建立后，继续把交接场景里最容易混淆的一层语义单独收紧：`receipt / acknowledgement`。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`shift handoff receipt and acknowledgement direction`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把交接场景从“有 follow-up”推进到“有独立 receipt 闭环”

## 上一阶段完成基线

上一阶段已完成：

- `follow-up / SLA` read model and query direction note
- task-scoped 与 cross-task read model 分层方向冻结
- 交接 receipt 与通用 follow-up 边界已进入明确设计问题

## 本阶段目标

1. 输出 `班次交接` 的 receipt / acknowledgement direction note。
2. 明确 receipt、follow-up accepted owner 和 formal approval 的边界。
3. 说明 receipt 应先如何进入 task detail，再如何进入跨班次 query。
4. 同步 `README / roadmap / planning / changelog / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `docs/design/m1-shift-handoff-receipt-acknowledgement-direction.md`
- 项目状态同步

本阶段暂不交付：

- receipt 领域模型代码
- acknowledgement action API 代码
- 跨班次 receipt queue 实现
- `shift log / incident log` connector 代码

## 风险与注意事项

- 不能把 receipt 写成 approval
- 不能把“交接包已接收”写成“所有 follow-up 都已 accepted”
- 不能把 receipt 状态写成高风险事项已被妥善处理的证明

## 进入本阶段的理由

如果不把 receipt 单独收紧，交接场景后续实现就很容易在三种语义之间来回混淆：交接包是否发出、接收方是否确认接住、具体遗留事项是否真正有人接手。只有把这层方向写清，交接场景才可能形成真实闭环。

## 本阶段完成结果

- 已交付 shift handoff receipt / acknowledgement direction note
- 已明确 receipt、follow-up、approval 三类状态机的边界
- 已把下一步从“通用协同对象”推进到“交接专属闭环对象”

## 实现摘要

这一阶段最重要的变化，是平台开始承认交接场景除了 follow-up 之外，还需要一层独立的“交接包被接住了吗”的对象。这个对象不属于审批，也不等于 item owner 接受，而是交接协同本身的闭环证据。

## 验证记录

已完成验证：

- 新方向文档已进入 `docs/design`
- `README / roadmap / planning / changelog / progress / journal` 已同步
- 文档变更已通过 `git diff --check`

## 阶段收口结论

FA 现在已经不仅知道交接场景需要 follow-up，也开始知道交接闭环还需要独立的 receipt / acknowledgement 层。下一步最合理的动作是继续收敛 `告警分诊` 的 alert cluster / event-ingestion 方向，并开始评估最小 `follow_up_items` 读模型进入代码主链的切口。
