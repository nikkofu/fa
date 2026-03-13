# M1 Alert Cluster Draft Seeding for Alert Triage Kickoff

## 日期

2026-03-12

## 同步目的

在 `alert_cluster_drafts / alert_triage_summary` 空 schema 已经进入代码主链后，继续把它推进到真正有非空 draft 的最小受控实现，优先选择 `alert triage` 这条高频事件协同场景，避免 cluster draft 长期停留在“字段存在，但一直为空”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster draft seeding for alert triage`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是为 `alert triage` 请求受控生成第一条非空 `alert_cluster_draft`

## 上一阶段完成基线

上一阶段已完成：

- handoff receipt draft seeding for shift handoff
- `shift handoff` 请求已能返回 1 条 seeded `handoff_receipt`
- 下一层最值得推进的是让告警任务场景也出现对应的非空 cluster draft

## 本阶段目标

1. 为 `alert triage` 请求生成 1 条受控 `alert_cluster_draft`。
2. 让 `alert_triage_summary` 返回最小但非零的 cluster 汇总。
3. 保持其他场景默认行为不变，不引入 ingestion API 或独立 query API。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `alert triage` 请求的 seeded `alert_cluster_draft`
- `alert_triage_summary` 汇总更新
- orchestrator / API / sandbox-safe file mode 测试覆盖
- 项目状态同步

本阶段暂不交付：

- raw alert event ingestion API
- 独立 alert cluster query endpoint
- 通用 alert clustering 引擎
- alert cluster 专属审计事件

## 风险与注意事项

- 不能把受控单场景 seeding 误写成“所有 alert 已经自动聚类”
- 不能让新逻辑影响现有高风险异常主链或 shift handoff 主链
- 不能为了第一条 cluster draft 引入新的 connector、写接口或事件总线

## 进入本阶段的理由

如果 `alert_cluster_drafts` 一直只有空 schema，平台虽然知道告警任务需要 triage，但还无法正式表达“哪些告警信号已经形成需要分诊的异常簇”。先为 `alert triage` 场景生成一条受控 cluster draft，能最快验证 task detail 主链是否真正可用。

## 本阶段完成结果

- 已为 `alert triage` 请求生成 1 条 seeded `alert_cluster_draft`
- 已验证 orchestrator、API contract 和 sandbox-safe file mode 持久化仍成立
- 已把下一步从“让 alert cluster 非空”推进到“扩第二类高频 follow-up / alert 模式与 receipt acknowledgement”

## 实现摘要

这一阶段最重要的变化，是 FA 第一次不再只返回空的 `alert_cluster_drafts`，而是能在 `alert triage` 这条高频事件协同场景上返回结构化 draft，包括 source system、severity、source refs、triage label 和 escalation candidate。这说明告警 task detail 主链已经开始真正承接事件分诊对象。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅有 alert triage task detail schema，也开始为高频告警场景生成第一条真实 cluster draft。下一步最合理的动作，是扩第二条高频 follow-up 场景、补最小 receipt acknowledgement 切口，并开始把 alert cluster 从单模式 draft 推进到更丰富的 source/window 推断。
