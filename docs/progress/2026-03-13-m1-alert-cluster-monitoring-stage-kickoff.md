# M1 Alert Cluster Monitoring Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `GET /api/v1/alert-clusters` 已成立后，继续把它推进到最小 `alert cluster monitoring` 聚合读视图。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster monitoring`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 alert cluster queue 推进到最小 monitoring 视图

## 上一阶段完成基线

上一阶段已完成：

- `tasks/intake` 与 `tasks/{task_id}` 已稳定返回 `alert_cluster_drafts / alert_triage_summary`
- `alert_cluster_drafts` 已支持 `source_system / line_id / triage_label / recommended_owner_role / cluster window` richer inference
- `GET /api/v1/alert-clusters` 已可跨任务返回 alert cluster queue

## 本阶段目标

1. 增加最小 `GET /api/v1/alert-cluster-monitoring` 聚合读接口。
2. 继续复用 `GET /api/v1/alert-clusters` 的过滤语义，不引入 monitor 专属 query 参数。
3. 输出最小 cluster backlog 汇总字段与 bucket 统计，覆盖 status、source、severity、triage label、owner、window state、risk、priority 八个视角。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `AlertClusterMonitoringView / AlertClusterMonitoringBucket`
- `WorkOrchestrator::get_alert_cluster_monitoring(...)`
- `GET /api/v1/alert-cluster-monitoring`
- monitoring API 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- dedicated alert-cluster projection 表
- cluster aging trend history / timeline snapshot
- cluster-to-follow-up linkage 读视图
- 真实 ingestion API 或 cluster monitor worker

## 风险与注意事项

- 不能让 monitoring 和 queue 出现不同的过滤口径
- 不能把第一版 monitoring 夸大成完整的事件协同大屏
- 不能为了 monitoring 提前引入新的 projection 存储

## 进入本阶段的理由

queue 解决的是“当前有哪些 cluster”，但值班现场更高频的问题很快会变成“这些 cluster 的整体轮廓是什么，哪些仍在活动窗口里，哪些已经过了窗口还没处理”。先补最小 monitoring 视图，能最快验证 alert cluster queue 是否已经足够支撑运行监控层。

## 本阶段完成结果

- 已为 alert cluster queue 增加最小 monitoring 聚合视图
- 已验证 monitoring 可返回核心 backlog 汇总字段和 bucket 统计
- 已验证 monitoring 复用 `alert-clusters` 的过滤语义
- 已验证 file-backed sandbox 模式重启后仍可回读 monitoring 结果
- 已验证现有 alert triage task detail、alert cluster queue、follow-up queue / monitoring、handoff receipt queue / monitoring 与高风险主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 不再只会列出 alert cluster backlog，而开始回答“当前异常簇 backlog 的形状是什么”。这说明 alert triage 能力开始从 queue 查询走向运行监控层。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能跨任务返回 alert cluster queue，也能在同一组过滤语义下返回最小 monitoring 视图。下一步更合理的动作，是评估何时把 repository-scan monitor 推进到 dedicated projection / backlog aging slices，并把 cluster 与 follow-up 联动起来。
