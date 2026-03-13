# M1 Handoff Receipt Escalation Stage Kickoff

## 日期

2026-03-12

## 同步目的

在 `shift handoff` receipt 已能通过显式 acknowledgement 进入 `acknowledged_with_exceptions` 后，继续把最小治理闭环从“异常已被接收方指出”推进到“发送侧 accountable 角色已明确升级处理”。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`handoff receipt escalation`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是为 `shift handoff` receipt 增加最小显式 escalation action

## 上一阶段完成基线

上一阶段已完成：

- `shift handoff` receipt 已拥有显式 `handoff-receipt/acknowledge` action
- `published -> acknowledged / acknowledged_with_exceptions` 状态迁移已可验证
- `receiving_role` 强校验与 `handoff_published / handoff_acknowledged / handoff_acknowledged_with_exceptions` 审计基线已成立
- 下一层最值得推进的是把 `acknowledged_with_exceptions` 继续落到 review / escalation 的最小治理动作

## 本阶段目标

1. 为 `shift handoff` receipt 增加显式 `handoff-receipt/escalate` action。
2. 只允许 `acknowledged_with_exceptions -> escalated`。
3. 对发送侧 accountable 角色做强校验，并记录 `handoff_receipt_escalated` audit event。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `handoff-receipt/escalate` action
- `handoff_receipt / handoff_receipt_summary` 状态更新
- `handoff_receipt_escalated` audit baseline
- orchestrator / API / sandbox-safe file mode 测试覆盖
- 项目状态同步

本阶段暂不交付：

- receipt overdue projection
- cross-shift escalation queue
- follow-up owner acceptance action
- item-level dispute resolution workflow

## 风险与注意事项

- 不能把 escalation action 错配成审批动作
- 不能让接收方异常说明自动等于“已经完成治理”
- 不能为了第一条 escalation action 引入过重的新 projection 或 queue

## 进入本阶段的理由

如果 `acknowledged_with_exceptions` 只能停留在 receipt 状态上，平台仍然回答不了“谁对异常交接包负起后续处理责任”。先补一条显式 escalation action，能最快验证 task detail、role guard、audit 和 file-backed persistence 是否足以支撑最小 review / escalation 切口。

## 本阶段完成结果

- 已为 `shift handoff` receipt 增加显式 escalation action
- 已验证 `acknowledged_with_exceptions -> escalated` 成立
- 已验证错误状态与错误角色会被拒绝
- 已验证 sandbox-safe file mode 重启后仍可回读 escalated receipt 与对应 audit event

## 实现摘要

这一阶段最重要的变化，是 FA 第一次不再只停在“接收方已指出问题”，而是允许发送侧 accountable 角色显式把异常交接包推进到 `escalated`。这说明交接 task detail 主链已经开始承接真正的 review / escalation 治理语义，而不只是展示接收结果。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能确认 `shift handoff` receipt 是否被接住，也能把“接住但有异议”的交接包推进到显式升级处理。下一步最合理的动作，是继续推进 follow-up owner acceptance action，并评估 handoff receipt overdue / cross-shift queue。
