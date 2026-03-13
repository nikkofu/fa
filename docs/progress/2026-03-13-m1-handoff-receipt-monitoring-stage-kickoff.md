# M1 Handoff Receipt Monitoring Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `cross-shift receipt queue` 已成立后，继续把它推进到最小 `receipt monitoring` 聚合读视图。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`handoff receipt monitoring`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 cross-shift receipt queue 推进到最小 monitoring 视图

## 上一阶段完成基线

上一阶段已完成：

- `GET /api/v1/handoff-receipts` 已可跨任务返回 cross-shift receipt queue
- queue 已支持 `shift_id / receipt_status / receiving_role / receiving_actor_id / overdue_only / has_exceptions / escalated_only` 过滤
- `GET /api/v1/follow-up-monitoring` 已可在同层返回最小 follow-up SLA monitoring

## 本阶段目标

1. 增加最小 `GET /api/v1/handoff-receipt-monitoring` 聚合读接口。
2. 继续复用 `GET /api/v1/handoff-receipts` 的过滤语义，不引入 monitor 专属 query 参数。
3. 输出最小 receipt backlog 汇总字段与 bucket 统计，覆盖 effective status、receiving role、ack window、risk、priority 五个视角。
4. 补 orchestrator / API / sandbox-safe file mode 测试覆盖，并同步文档。

## 本阶段交付边界

本阶段计划交付：

- `HandoffReceiptMonitoringView / HandoffReceiptMonitoringBucket`
- `WorkOrchestrator::get_handoff_receipt_monitoring(...)`
- `GET /api/v1/handoff-receipt-monitoring`
- monitoring API 测试与 sandbox smoke
- 项目状态同步

本阶段暂不交付：

- dedicated receipt projection 表
- aging trend history / timeline snapshot
- receipt reassignment 或 exception resolution 新写接口
- 独立 receipt monitor worker

## 风险与注意事项

- 不能让 monitoring 和 receipt queue 出现不同的过滤口径
- 不能把 `expired` 夸大成正式写入状态机
- 不能为了第一版 monitoring 提前引入新的 projection 存储

## 进入本阶段的理由

班次交接值班不只关心“有哪些交接包”，更关心“还有多少包没接住、哪些包已经带异议、哪些包已经到了升级边界”。先补最小 monitoring 视图，能最快验证 receipt queue 是否已经足够支撑运行监控层。

## 本阶段完成结果

- 已为 receipt queue 增加最小 monitoring 聚合视图
- 已验证 monitoring 可返回核心 backlog 汇总字段和 bucket 统计
- 已验证 monitoring 复用 `handoff-receipts` 的过滤语义
- 已验证 file-backed sandbox 模式重启后仍可回读 monitoring 结果
- 已验证现有 task detail、follow-up monitoring、receipt queue 与 alert triage 主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 不再只会列出 cross-shift receipt queue，而开始回答“当前 receipt backlog 的整体形状是什么”。这说明平台的交接协同能力开始从 queue 查询走向运行监控层。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能跨班次返回 receipt queue，也能在同一组过滤语义下返回最小 monitoring 视图。下一步更合理的动作，是评估何时把 repository-scan monitor 推进到 dedicated projection / aging trend slices。
