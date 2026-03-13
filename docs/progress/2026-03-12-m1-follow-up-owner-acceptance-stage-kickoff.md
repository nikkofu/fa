# M1 Follow-up Owner Acceptance Stage Kickoff

## 日期

2026-03-12

## 同步目的

在 `shift handoff` 与 `alert triage` 已能稳定返回 seeded `follow_up_item` draft 后，继续把最小高频协同闭环从“系统推荐谁接手”推进到“责任角色已显式接手”。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up owner acceptance`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是为 task-scoped seeded `follow_up_item` 增加最小显式 owner acceptance action

## 上一阶段完成基线

上一阶段已完成：

- `shift handoff` receipt 已拥有显式 acknowledgement 与 escalation action
- `follow_up_items` 已能在 `shift handoff` 与 `alert triage` 中返回非空 seeded draft
- 下一层最值得推进的是把 recommended owner 与 accepted owner 的差异落到显式 action

## 本阶段目标

1. 为 task-scoped seeded `follow_up_item` 增加显式 `follow-up-items/{follow_up_id}/accept-owner` action。
2. 只允许 `draft -> accepted`。
3. 对 `recommended_owner_role` 做强校验，并记录 `follow_up_owner_accepted` audit event。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `follow-up-items/{follow_up_id}/accept-owner` action
- `follow_up_items / follow_up_summary` 状态更新
- `handoff_receipt_summary.unaccepted_follow_up_count` 联动更新
- `follow_up_owner_accepted` audit baseline
- orchestrator / API / sandbox-safe file mode 测试覆盖
- 项目状态同步

本阶段暂不交付：

- follow-up item assignment action
- cross-task owner queue
- overdue / escalation projection
- item-level completion / blocked / reassign state machine

## 风险与注意事项

- 不能把 accepted owner 和 handoff receipt acknowledgement 混成一个动作
- 不能把 recommended owner 自动写成 accepted owner
- 不能为了第一条 acceptance action 引入过重的 item-level aggregate 或 projection

## 进入本阶段的理由

如果 `follow_up_item` 一直停留在 seeded draft，平台仍然回答不了“系统建议谁接手”和“谁真的接手了”之间的差异。先补一条显式 acceptance action，能最快验证 task detail、role guard、audit 和 file-backed persistence 是否足以支撑最小 owner acceptance 闭环。

## 本阶段完成结果

- 已为 task-scoped seeded `follow_up_item` 增加显式 owner acceptance action
- 已验证 `draft -> accepted` 成立
- 已验证错误角色与重复 acceptance 会被拒绝
- 已验证 `shift handoff` owner acceptance 后 `handoff_receipt_summary.unaccepted_follow_up_count` 会同步收敛
- 已验证 sandbox-safe file mode 重启后仍可回读 accepted follow-up 与对应 audit event

## 实现摘要

这一阶段最重要的变化，是 FA 第一次不再只停在“系统建议下一步谁来做”，而是允许责任角色显式接手该条待办。这说明 follow-up task detail 主链已经开始承接真正的 owner acceptance 语义，而不只是展示建议。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能在任务详情中推荐 follow-up owner，也能显式表达“这条待办已被谁接住”。下一步最合理的动作，是继续推进 assignment / cross-task owner queue，并补 due / overdue 视图。
