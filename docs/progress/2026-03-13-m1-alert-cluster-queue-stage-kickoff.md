# M1 Alert Cluster Queue Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `alert_cluster_drafts` 已完成 richer inference 之后，把它推进到最小 cross-task `alert cluster` queue 读层。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster queue`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是补最小 cross-task `alert cluster` queue，而不引入新写模型

## 上一阶段完成基线

上一阶段已完成：

- `tasks/intake` 与 `tasks/{task_id}` 已稳定返回 `alert_cluster_drafts / alert_triage_summary`
- `alert_cluster_drafts` 已支持 `source_system / line_id / triage_label / recommended_owner_role / cluster window` richer inference
- `alert triage` 已支持 `andon` 的 `repeated_alert_review` 与 `scada` 的 `sustained_threshold_review` 两类 draft 形态

## 本阶段目标

1. 增加最小 `GET /api/v1/alert-clusters` cross-task queue 读接口。
2. 让 queue 直接复用 repository-scan 读层，不引入 dedicated projection 表。
3. 支持 `cluster_status / source_system / equipment_id / line_id / severity_band / triage_label / escalation_candidate / window_from / window_to / open_only` 过滤。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `AlertClusterQueueQuery / AlertClusterQueueItemView`
- `WorkOrchestrator::list_alert_clusters(...)`
- `GET /api/v1/alert-clusters`
- queue API 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- dedicated alert-cluster projection 表
- `alert cluster monitoring` 聚合视图
- cluster 到 follow-up / SLA 的联动读视图
- 真实 ingestion API 或 event normalization worker

## 风险与注意事项

- 不能把 `alert cluster` queue 和任务详情里的 draft 语义分叉
- 不能为了第一版 queue 提前引入新的 projection 存储
- `window_from / window_to` 需要保持明确可解释的 overlap 过滤口径

## 进入本阶段的理由

task detail 能回答“单个 triage task 里有哪些 cluster”，但现场更高频的问题开始变成“当前所有产线异常簇里，哪些最该优先看”。先补一个最小 cross-task queue，能验证 alert cluster 是否已经形成真正的平台级 backlog 对象。

## 本阶段完成结果

- 已为 alert cluster 增加最小 cross-task queue 读视图
- 已验证 queue 可返回 task metadata、cluster core fields 与默认优先级排序
- 已验证 queue 支持来源、产线、triage 形态、升级候选与时间窗过滤
- 已验证 file-backed sandbox 模式重启后仍可回读 queue 与过滤结果
- 已验证现有 alert triage task detail、follow-up queue / monitoring、handoff receipt queue / monitoring 与高风险主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 不再只在任务详情里持有 `alert_cluster_draft`，而开始把它暴露成跨任务 backlog 视图。平台因此第一次具备了“当前有哪些异常簇正在堆积、哪些需要先处理”的最小答案。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能在任务详情里展示 alert cluster draft，也能跨任务返回最小 alert cluster queue。下一步更合理的动作，是评估何时把 queue 推进到 monitoring / follow-up linkage 或 dedicated projection。
