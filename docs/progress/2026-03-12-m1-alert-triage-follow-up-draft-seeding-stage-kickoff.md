# M1 Alert Triage Follow-up Draft Seeding Kickoff

## 日期

2026-03-12

## 同步目的

在 `alert triage` 已经能生成非空 `alert_cluster_draft` 后，继续把 `follow_up_items` 从单一 `shift handoff` 场景扩到第二条高频主线，避免告警分诊长期停留在“有 cluster，但没有明确 owner-action draft”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert triage follow-up draft seeding`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是为 `alert triage` 请求受控生成第二条高频 `follow_up_item`

## 上一阶段完成基线

上一阶段已完成：

- alert cluster draft seeding for alert triage
- `alert triage` 请求已能返回 1 条 seeded `alert_cluster_draft`
- 下一层最值得推进的是让同一场景出现对应的 owner-action follow-up draft

## 本阶段目标

1. 为 `alert triage` 请求生成 1 条受控 `follow_up_item` draft。
2. 保持同一请求下的 `alert_cluster_draft` 仍然稳定存在。
3. 保持其他场景默认行为不变，不引入 owner acceptance / assignment action。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `alert triage` 请求的 seeded `follow_up_item`
- `follow_up_summary` 汇总更新
- orchestrator / API / sandbox-safe file mode 测试覆盖
- 项目状态同步

本阶段暂不交付：

- follow-up owner acceptance action
- follow-up assignment write API
- 第二类 alert triage pattern
- 独立 follow-up queue 或 cross-task query API

## 风险与注意事项

- 不能把受控单场景 seeding 误写成“所有 follow-up 已经通用自动生成”
- 不能让新的 follow-up 逻辑破坏已有 `shift handoff` 或默认高风险异常主链
- 不能为了第二条 follow-up draft 引入新的 connector、action endpoint 或状态机

## 进入本阶段的理由

如果 `alert triage` 一直只有 cluster，没有 follow-up draft，平台虽然能回答“哪些告警聚成了一簇”，却还不能回答“谁应该立刻接住这簇告警并做第一步响应”。先让同一请求下同时出现 cluster 和 follow-up，能最快验证 task detail 主链是否真正承接了事件到行动的衔接。

## 本阶段完成结果

- 已为 `alert triage` 请求生成 1 条 seeded `follow_up_item`
- 已验证同一请求下 cluster 与 follow-up 可同时回读
- 已把下一步从“扩第二条高频 follow-up 场景”推进到“owner acceptance / receipt acknowledgement / richer alert inference”

## 实现摘要

这一阶段最重要的变化，是 FA 第一次能在 `alert triage` 这条高频事件协同场景上同时返回结构化 `alert_cluster_draft` 和结构化 `follow_up_item`。这说明任务详情主链已经不只是表达异常簇，也开始表达面向角色分派的后续动作草稿。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅能在 `shift handoff` 返回 follow-up，也能在 `alert triage` 返回第二条高频 follow-up draft。下一步最合理的动作，是补最小 owner acceptance / receipt acknowledgement 切口，并继续丰富 alert cluster 的 source/window 推断。
