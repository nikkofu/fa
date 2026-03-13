# M1 Alert Cluster Inference Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `alert_cluster_drafts` 已经进入 task detail 后，继续把它推进到更接近真实现场的 richer inference，而不是立刻跳到独立 queue 或 ingestion API。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster inference`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 task-scoped alert cluster 从单一 seed 推进到 richer source / line / window inference

## 上一阶段完成基线

上一阶段已完成：

- `alert_cluster_drafts / alert_triage_summary` 已稳定进入 `tasks/{task_id}`
- `shift handoff` receipt queue / monitoring 已形成连续读层
- `follow-up` queue / monitoring 已形成连续读层

## 本阶段目标

1. 增强 `alert_cluster_draft` 的 `source_system / line_id / triage_label / recommended_owner_role / cluster window` 推断。
2. 为 `alert triage` 增加第二类 `sustained_threshold_review` draft 形态。
3. 保持改动仍停留在 task-scoped read model，不引入新 route 或 ingestion adapter。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- richer alert cluster inference helpers
- second `scada threshold` draft shape
- API / smoke / QA 覆盖
- 项目状态同步

本阶段暂不交付：

- `GET /api/v1/alert-clusters`
- raw alert ingestion API
- `Scada / Andon` mock connector
- dedicated cluster projection 表

## 风险与注意事项

- 不能把 richer inference 误写成 ingestion 主线
- 不能让 cluster draft 和 follow-up owner 路由出现自相矛盾
- 不能为了第二类 alert mode 把第一类 `andon` 形态回退

## 进入本阶段的理由

如果 cluster draft 永远只有一类固定 seed，它就还不能真正表达现场差异。先补 richer inference，能最快验证当前 task-scoped read model 是否足以承接第二类高频告警形态，而不必先引入更重的事件系统。

## 本阶段完成结果

- 已为 alert cluster draft 增加 richer source / line / window inference
- 已为 `scada` 阈值类告警增加第二类 `sustained_threshold_review` 形态
- 已验证 file-backed sandbox 模式重启后仍可回读 richer inference 结果
- 已验证现有 `andon` 告警形态和其他主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 的 alert triage 不再只有一类“重复告警”草稿，而开始区分 `andon burst` 和 `scada threshold drift` 两类现场事件形态。这说明 alert triage 开始从静态示例走向更真实的运行语义。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能在 task detail 中返回 `alert_cluster_drafts`，也能更合理地推断第二类高频告警形态。下一步更合理的动作，是评估何时把 richer task-scoped draft 推进到 cross-task `alert cluster` queue。
