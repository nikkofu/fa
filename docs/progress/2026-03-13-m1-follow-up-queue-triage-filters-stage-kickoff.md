# M1 Follow-up Queue Triage Filters Stage Kickoff

## 日期

2026-03-13

## 同步目的

在 `GET /api/v1/follow-up-items` 已经能返回最小 cross-task owner queue 后，继续把它推进到更接近现场值班与异常分诊的运营 triage 视图。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up queue triage filters`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 owner queue 扩成更接近现场值班问题的 triage queue

## 上一阶段完成基线

上一阶段已完成：

- `GET /api/v1/follow-up-items` 已能跨任务聚合 follow-up owner queue
- queue 已支持 `owner_id / source_kind / status / overdue_only` 等最小过滤
- 下一层最值得推进的是运营 triage 过滤，而不是先开新的 projection 表

## 本阶段目标

1. 为 `GET /api/v1/follow-up-items` 增加 `blocked_only / escalation_required / due_before / risk / priority` 过滤。
2. 继续复用 repository 扫描与现有 `FollowUpQueueItemView`，不引入新的存储层。
3. 补 orchestrator / API / sandbox-safe file mode 测试覆盖。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `FollowUpQueueQuery` 扩展
- `GET /api/v1/follow-up-items` triage filters
- 内存 / 文件模式 triage query 自动化验证
- 项目状态同步

本阶段暂不交付：

- dedicated follow-up projection 表
- blocked / escalation item 写接口
- item completion / reassign / aging 视图
- 独立 SLA monitoring endpoint

## 风险与注意事项

- 不能把 triage filters 误做成 item 状态机扩容
- 不能为了过滤能力扩一批不存在的写接口
- 不能让 query 语义脱离 `follow_up_item` 现有正式字段

## 进入本阶段的理由

如果 queue 只能回答“谁手里有哪些待办”，但不能回答“哪些是 blocked、哪些要 escalation、哪些更高风险更高优先级”，那它仍然不够贴近现场值班和日常异常分诊。先补 triage filters，能最快验证现有读模型是否足够承接更高频的运营筛选问题。

## 本阶段完成结果

- 已为 queue 增加 `blocked_only / escalation_required / due_before / risk / priority` 过滤
- 已验证 triage filters 能把 `shift handoff` 与 `alert triage` follow-up 区分成更接近现场处理顺序的结果
- 已验证 file-backed sandbox 模式重启后仍可回读 triage filter 结果
- 已验证现有 owner acceptance、handoff receipt 与 alert triage 主链不受影响

## 实现摘要

这一阶段最重要的变化，是 FA 的 follow-up queue 开始不只服务“看 backlog”，还开始服务“先处理什么、哪些需要盯紧”。这说明平台的高频协同能力正在从 owner read 走向 operational triage read。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`
- `git diff --check`

## 阶段收口结论

FA 现在已经不仅能跨任务返回 owner queue，也能按 blocked、escalation、risk、priority 和 due window 做最小 triage 过滤。下一步最合理的动作，是评估何时需要把这些读能力从 repository 扫描推进到 dedicated projection / SLA monitoring view。
