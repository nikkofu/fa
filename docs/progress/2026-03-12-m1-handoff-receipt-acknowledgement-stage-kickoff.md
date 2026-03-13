# M1 Handoff Receipt Acknowledgement Stage Kickoff

## 日期

2026-03-12

## 同步目的

在 `shift handoff` 已经能返回非空 `handoff_receipt` draft 后，继续把交接闭环从“只读对象”推进到“显式 acknowledgement action”，避免 receipt 长期停留在“能看见，但系统无法确认是否已被接住”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`handoff receipt acknowledgement`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是为 `shift handoff` receipt 增加最小显式 acknowledgement action

## 上一阶段完成基线

上一阶段已完成：

- alert triage follow-up draft seeding
- `alert triage` 已能同时返回 cluster draft 和 follow-up draft
- 下一层最值得推进的是让 `shift handoff` receipt 从只读 draft 进入真正可确认状态

## 本阶段目标

1. 为 `shift handoff` receipt 增加显式 `handoff-receipt/acknowledge` action。
2. 只允许 `published -> acknowledged / acknowledged_with_exceptions`。
3. 对 `receiving_role` 做强校验，并记录 handoff audit event。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `handoff-receipt/acknowledge` action
- `handoff_receipt / handoff_receipt_summary` 状态更新
- `handoff_published / handoff_acknowledged / handoff_acknowledged_with_exceptions` audit baseline
- orchestrator / API / sandbox-safe file mode 测试覆盖
- 项目状态同步

本阶段暂不交付：

- receipt overdue projection
- receipt escalation queue
- follow-up owner acceptance action
- 跨班次 handoff receipt query API

## 风险与注意事项

- 不能把 acknowledgement action 错配成审批动作
- 不能把 receipt acknowledged 误写成所有 follow-up 已被 accepted
- 不能为了第一条 acknowledgement action 引入过重的新状态机或 projection

## 进入本阶段的理由

如果 receipt 一直只能被读取，就无法回答“接收班次是否已经正式接住这份交接包”。先补一条显式 acknowledgement action，能最快验证 task detail、role guard、audit 和 file-backed persistence 是否足以支撑最小交接闭环。

## 本阶段完成结果

- 已为 `shift handoff` receipt 增加显式 acknowledgement action
- 已验证 receipt 可进入 `acknowledged / acknowledged_with_exceptions`
- 已验证 role mismatch 会被拒绝
- 已把下一步从“能否确认 receipt”推进到“ack exception 后如何进入 review / escalation”

## 实现摘要

这一阶段最重要的变化，是 FA 第一次不再只返回 `handoff_receipt` draft，而是允许接收角色通过显式 action 正式确认交接包。这说明交接 task detail 主链已经开始承接真正的接收闭环，而不只是展示一份静态对象。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅能表达 `shift handoff` receipt，也能显式确认 receipt 是否被接住。下一步最合理的动作，是为 `acknowledged_with_exceptions` 增加 review / escalation 治理切口，并继续推进 follow-up owner acceptance action。
