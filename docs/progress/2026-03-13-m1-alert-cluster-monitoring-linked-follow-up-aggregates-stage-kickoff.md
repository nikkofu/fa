# M1 Alert Cluster Monitoring Linked Follow-up Aggregates Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `GET /api/v1/alert-cluster-monitoring` 已支持 linked follow-up triage filters 后，继续把监控视图推进到最小 linked follow-up backlog aggregate 能力。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster monitoring linked follow-up aggregates`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是让 monitoring 直接回答 linked follow-up backlog 的总体形态

## 上一阶段完成基线

上一阶段已完成：

- `GET /api/v1/alert-clusters` 已支持 linked follow-up triage filters
- `GET /api/v1/alert-cluster-monitoring` 已复用相同的 linked follow-up filter 口径
- file-backed sandbox 模式重启后已验证可回读 linked follow-up triage filter 结果

## 本阶段目标

1. 为 `GET /api/v1/alert-cluster-monitoring` 增加 linked follow-up backlog aggregate 字段。
2. 保持 queue / monitoring 口径一致，不引入新的 route 或 projection。
3. 先覆盖最小高频监控问题：是否已挂上 follow-up、是否有人接、是否仍有未接单 backlog、是否已经进入升级态。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `linked_follow_up_clusters`
- `unlinked_follow_up_clusters`
- `accepted_follow_up_clusters`
- `unaccepted_follow_up_clusters`
- `follow_up_escalation_clusters`
- `follow_up_coverage_counts`
- `follow_up_sla_status_counts`
- monitoring API 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- accepted owner load / owner workload bucket
- dedicated projection / join table
- cluster dashboard 专属 endpoint
- cluster follow-up 写侧建模

## 风险与注意事项

- 不能让 monitoring aggregate 和 queue linkage 语义脱钩
- 不能把第一版 aggregate 扩大成完整运营大盘
- 不能为了 aggregate 提前引入新的 projection 存储

## 进入本阶段的理由

triage filter 解决的是“先筛哪一类 cluster”，但主管下一步仍然会问“整体 backlog 里究竟多少 cluster 已挂上 follow-up、多少已有人接、多少还没人接、多少已经升级”。如果 monitoring 不能直接回答这些问题，系统还停留在列表过滤层，而不是监控层。

## 本阶段完成结果

- 已为 `GET /api/v1/alert-cluster-monitoring` 增加 linked follow-up backlog aggregate 字段
- 已验证 monitoring 可直接返回 linked/unlinked/accepted/unaccepted/escalation 五组 linked follow-up 汇总字段
- 已验证 monitoring 可返回 `follow_up_coverage / follow_up_sla_status` 两组 linked follow-up bucket
- 已验证 file-backed sandbox 模式重启后仍可回读这些 monitoring aggregate 字段
- 已验证现有 alert triage task detail、alert cluster queue / linkage / triage filters、follow-up queue / monitoring、handoff receipt queue / monitoring 与高风险主链不受影响

## 实现摘要

这一阶段最重要的变化，是 alert cluster monitoring 开始直接回答 “linked follow-up backlog 现在是什么形态”，而不是只告诉用户 cluster 本身的 severity、status 和 window state。这样 monitoring 视图就从事件簇监控层进一步靠近异常处置运营层。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经能在 `alert-cluster-monitoring` 上直接回答 linked follow-up backlog 的覆盖率、接单状态和升级态。下一步更合理的动作，是评估是否增加 accepted owner / owner-load 聚合维度，并在查询频率足够高时再推进到 dedicated projection。
