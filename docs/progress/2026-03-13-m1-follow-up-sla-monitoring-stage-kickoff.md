# M1 Follow-up SLA Monitoring Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `follow-up owner queue` 与 triage filters 已成立后，继续把它推进到最小 `SLA monitoring` 聚合读视图。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up SLA monitoring`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 cross-task follow-up queue 推进到最小 monitoring 视图

## 上一阶段完成基线

上一阶段已完成：

- `GET /api/v1/follow-up-items` 已可跨任务返回 follow-up owner queue
- queue 已支持 `blocked_only / escalation_required / due_before / risk / priority` triage filters
- `GET /api/v1/handoff-receipts` 已可跨任务返回 cross-shift receipt queue

## 本阶段目标

1. 增加最小 `GET /api/v1/follow-up-monitoring` 聚合读接口。
2. 继续复用 `GET /api/v1/follow-up-items` 的过滤语义，不引入新的 monitor 专属筛选条件。
3. 输出最小 backlog 汇总字段与 bucket 统计，覆盖 owner、source、SLA、risk、priority 五个视角。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `FollowUpMonitoringView / FollowUpMonitoringBucket`
- `WorkOrchestrator::get_follow_up_monitoring(...)`
- `GET /api/v1/follow-up-monitoring`
- monitoring API 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- dedicated follow-up projection 表
- backlog aging 历史快照
- item completion / reassignment 新写接口
- 独立 SLA worker 或 monitor job

## 风险与注意事项

- 不能让 monitoring 与 owner queue 出现不同的过滤语义
- 不能为了第一版聚合视图提前引入新的 projection 存储
- 不能把 monitoring 误写成手工维护状态，而应继续保持 read-only 聚合

## 进入本阶段的理由

现场协同只知道“有哪些待办”还不够，更高频的问题是“哪些 backlog 已经堆积、哪些角色还没接单、哪些事项已经到升级边界”。先补最小 monitoring 视图，能最快验证当前 queue 语义是否足以支撑运行监控层。

## 本阶段完成结果

- 已为 follow-up queue 增加最小 SLA monitoring 聚合视图
- 已验证 monitoring 可返回核心 backlog 汇总字段和 bucket 统计
- 已验证 monitoring 复用 `follow-up-items` 的过滤语义
- 已验证 file-backed sandbox 模式重启后仍可回读 monitoring 结果
- 已验证现有 task detail、follow-up queue、handoff receipt queue 与 alert triage 主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 不再只会枚举 cross-task follow-up items，而开始回答“当前 backlog 的整体形状是什么”。这说明平台的 follow-up 能力开始从 owner queue 走向运行监控层。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能跨任务返回 follow-up queue，也能在同一组过滤语义下返回最小 monitoring 视图。下一步更合理的动作，是评估何时把 repository-scan monitor 推进到 dedicated projection / backlog aging slices。
