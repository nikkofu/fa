# M1 Follow-up Draft Seeding for Shift Handoff Kickoff

## 日期

2026-03-12

## 同步目的

在 `follow_up_items / follow_up_summary` 空 schema 已经进入代码主链后，继续把它推进到真正有非空 draft 的最小受控实现，优先选择 `shift handoff` 这条高频场景，避免 follow-up 长期停留在“字段存在，但一直为空”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up draft seeding for shift handoff`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是为 `shift handoff` 请求受控生成第一条非空 `follow_up_item`

## 上一阶段完成基线

上一阶段已完成：

- alert triage task detail schema cut
- 高频 task-detail trio 的空默认 schema 已全部进入代码主链
- 下一层最值得推进的是让至少一条高频场景真正产生非空 draft

## 本阶段目标

1. 为 `shift handoff` 请求生成 1 条受控 `follow_up_item` draft。
2. 保持其他场景默认行为不变，不引入新的写接口或新 connector。
3. 补齐 orchestrator、API contract 和 sandbox smoke 相关验证。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `shift handoff` 请求的 seeded `follow_up_item`
- `follow_up_summary` 汇总更新
- orchestrator / API 测试覆盖
- 项目状态同步

本阶段暂不交付：

- 通用 follow-up 自动生成引擎
- `handoff_receipt` draft 生成
- `alert_cluster_drafts` draft 生成
- follow-up item 写接口或跨任务 queue

## 风险与注意事项

- 不能把受控单场景 seeding 误写成“所有 follow-up 已经智能生成”
- 不能让新逻辑影响现有高风险异常主链
- 不能为了生成 draft 去引入新的 connector 或新的 endpoint

## 进入本阶段的理由

如果 follow-up 一直只有空 schema，平台虽然 contract 更完整了，但还看不出真实协同对象如何被接住。先为 `shift handoff` 这条高频、低风险场景生成一条受控 draft，能最快验证 task detail 主链是否真正可用。

## 本阶段完成结果

- 已为 `shift handoff` 请求生成 1 条 seeded `follow_up_item`
- 已验证 orchestrator、API contract 和 sandbox smoke 仍成立
- 已把下一步从“让 follow-up 非空”推进到“handoff receipt / alert cluster 也开始出现非空 draft”

## 实现摘要

这一阶段最重要的变化，是 FA 第一次不再只返回空的 `follow_up_items`，而是能在 `shift handoff` 这条高频场景上返回结构化 draft，包括来源类型、建议角色、到期时间和汇总状态。这说明任务详情主链已经开始真正承接后续协同对象。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅有 follow-up task detail schema，也开始为高频场景生成第一条真实 draft。下一步最合理的动作，是继续让 `handoff_receipt` 和 `alert_cluster_drafts` 出现对应的非空 draft，并评估 `QMS` baseline 的代码切口。
