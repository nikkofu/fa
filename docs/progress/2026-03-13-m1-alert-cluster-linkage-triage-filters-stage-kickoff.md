# M1 Alert Cluster Linkage Triage Filters Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `GET /api/v1/alert-clusters` 已能返回 `linked_follow_up` 摘要后，继续把 cluster backlog 推进到最小 linked follow-up triage filter 能力。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`alert cluster linkage triage filters`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是让 cluster queue / monitoring 能按 linked follow-up 运营状态直接过滤

## 上一阶段完成基线

上一阶段已完成：

- `GET /api/v1/alert-clusters` queue item 已可返回 `linked_follow_up` 摘要
- queue 已可直接暴露 follow-up 总量、accepted owner 和最高优先级 SLA 状态
- file-backed sandbox 模式重启后已验证可回读 linked follow-up 摘要

## 本阶段目标

1. 为 `GET /api/v1/alert-clusters` 增加 linked follow-up triage 过滤。
2. 让 `GET /api/v1/alert-cluster-monitoring` 复用同一组过滤语义。
3. 优先支持最小高频维度：accepted owner、仍未接单、已升级 follow-up。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `follow_up_owner_id`
- `unaccepted_follow_up_only`
- `follow_up_escalation_required`
- queue / monitoring 过滤测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- 新的 cluster monitoring bucket
- linked follow-up 聚合大盘
- dedicated projection / join table
- cluster follow-up 专属 endpoint

## 风险与注意事项

- 不能让 monitoring 和 queue 在 linked follow-up 过滤上出现口径漂移
- 不能把 triage filter 扩大成新的 cluster 生命周期状态机
- 不能让 linked follow-up filter 误命中非 alert 场景数据

## 进入本阶段的理由

现场最常见的问题很快会从“这个 cluster 里有什么”变成“谁接了、哪些还没人接、哪些已经开始升级”。如果这些问题还要靠人工读每条 queue item，就说明平台距离真正的运营 triage 面板还差一步。

## 本阶段完成结果

- 已为 `GET /api/v1/alert-clusters` 增加 linked follow-up triage filters
- 已验证 queue 和 monitoring 都支持按 accepted owner、未接单状态和 escalation-required 状态过滤
- 已验证 file-backed sandbox 模式重启后仍可回读 triage filter 结果
- 已验证现有 alert triage task detail、alert cluster linkage、follow-up queue / monitoring、handoff receipt queue / monitoring 与高风险主链不受影响

## 实现摘要

这一阶段最重要的变化，是 alert cluster backlog 开始可以按真实处置状态直接筛选，而不再要求主管逐条读 JSON 里的 linked follow-up 摘要。这让 cluster queue 更接近一线实际使用的待办分诊入口。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经能把 alert cluster backlog 按 accepted owner、未接单状态和 follow-up 升级态直接过滤。下一步更合理的动作，是评估何时把 linked follow-up triage 维度推进到更显式的 monitoring 聚合字段或 dedicated projection。
