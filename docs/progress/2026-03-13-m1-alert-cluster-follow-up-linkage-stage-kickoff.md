# M1 Alert Cluster Follow-up Linkage Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `GET /api/v1/alert-clusters` queue 和 `GET /api/v1/alert-cluster-monitoring` 已成立后，继续把 cluster backlog 推进到最小 follow-up 联动读层。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster follow-up linkage`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是让 cluster queue 直接回答 follow-up ownership 和 SLA 问题

## 上一阶段完成基线

上一阶段已完成：

- `GET /api/v1/alert-clusters` 已可跨任务返回 alert cluster queue
- `GET /api/v1/alert-cluster-monitoring` 已可返回最小 cluster backlog 聚合视图
- queue 与 monitoring 已稳定复用同一组 cluster backlog 过滤语义

## 本阶段目标

1. 在 `GET /api/v1/alert-clusters` 的 queue item 上增加最小 linked follow-up 摘要。
2. 优先支持显式 `follow_up_item.source_kind=alert_cluster` 且 `source_refs` 指向 `cluster_id` 的联动。
3. 对现有单 cluster `alert triage` task 保持兼容回退，不打断已存在的 follow-up seeding / acceptance 路径。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `AlertClusterLinkedFollowUpView`
- `GET /api/v1/alert-clusters` queue item 上的 `linked_follow_up` 摘要
- cluster-to-follow-up 只读匹配逻辑
- queue linkage API 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- cluster 专属 follow-up read endpoint
- dedicated alert-cluster projection / join table
- cluster-level follow-up 写侧建模
- monitoring 聚合里的 linked follow-up bucket 统计

## 风险与注意事项

- 不能把 queue linkage 做成新的写侧状态机
- 不能误把非 alert 场景 follow-up 混到 cluster queue 里
- 兼容回退只能作为过渡方案，不能掩盖显式 `cluster_id` 引用的长期方向

## 进入本阶段的理由

对一线值班者来说，“有 cluster” 还不够，更高频的问题是“这个 cluster 到底有没有人接、现在 SLA 是否已经开始冒烟”。先把这层直接挂到 queue item 上，能最快把 alert triage 从异常列表推进到可执行的处置协同层。

## 本阶段完成结果

- 已为 `GET /api/v1/alert-clusters` queue item 增加最小 `linked_follow_up` 摘要
- 已验证 queue item 可直接返回 follow-up 总量、接单状态、accepted owner 和最高优先级 SLA 状态
- 已验证 linkage 优先支持显式 `cluster_id` 引用，并兼容单 cluster `alert_triage` 旧链路
- 已验证 file-backed sandbox 模式重启后仍可回读 linkage 结果
- 已验证现有 alert triage task detail、alert cluster monitoring、follow-up queue / monitoring、handoff receipt queue / monitoring 与高风险主链不受影响

## 实现摘要

这一阶段最重要的变化，是 alert cluster queue 不再只展示“簇本身”，而开始直接暴露“这个簇后续有没有被接住”。这让 cluster queue 从事件分诊列表进一步靠近现场处置工作台。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经能在 `alert-clusters` queue 上直接回答 cluster 是否已有 follow-up、谁接单了、最高优先级 SLA 状态是什么。下一步更合理的动作，是评估何时把 repository-scan linkage 推进到 dedicated projection / aging slices，并把 linked follow-up 聚合能力带入 monitoring 视图。
