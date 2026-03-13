# M1 Follow-up Task Read Model Schema Cut Kickoff

## 日期

2026-03-12

## 同步目的

在 `follow_up_items` implementation cut note 已经建立后，继续把它推进到真正进入代码主链的最小 schema cut，避免 task detail 一直停留在“知道该加什么字段，但 API 里还看不到”的状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`follow-up task read model schema cut`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`7b24ce1`
- 当前本地分支状态：继续在 `v0.2.0` 本地未提交演进上推进，本阶段重点是把 `follow_up_items / follow_up_summary` 以兼容旧任务 JSON 的方式真实接入 `TrackedTaskState`

## 上一阶段完成基线

上一阶段已完成：

- alert triage task read model implementation cut note
- 高频 task-detail trio 的 implementation cut 已全部形成
- 下一层最值得推进的是把 `follow_up_items` schema cut 从说明推进到代码

## 本阶段目标

1. 把 `follow_up_items / follow_up_summary` 加入 `TrackedTaskState`。
2. 保持 `tasks/{task_id}`、file repository、SQLite repository 和旧 JSON 的兼容。
3. 补齐 repository round-trip、API contract 和 sandbox smoke 断言。
4. 同步 `README / roadmap / changelog / qa / progress / journal`。

## 本阶段交付边界

本阶段计划交付：

- `TrackedTaskState.follow_up_items`
- `TrackedTaskState.follow_up_summary`
- compatibility / repository / API contract 测试与 smoke 断言
- 项目状态同步

本阶段暂不交付：

- 非空 `follow_up_items` 自动生成逻辑
- 跨任务 follow-up queue API
- follow-up item 写接口
- item 级专属审计事件

## 风险与注意事项

- 不能破坏旧任务 JSON 的反序列化
- 不能为了 schema cut 去改动 task lifecycle 或 route 结构
- 不能把默认空 schema 误写成“follow-up 业务逻辑已完整实现”

## 进入本阶段的理由

如果 implementation cut 一直不进入代码，FA 仍然只能返回 task、evidence 和 governance，而不能正式表达后续协同对象。只有先把 `follow_up_items / follow_up_summary` 接进任务详情，后续高频协同对象才真正有了平台主链入口。

## 本阶段完成结果

- 已交付最小 `follow_up_items / follow_up_summary` task detail schema cut
- 已验证旧 JSON 兼容、repository round-trip、API contract 与 sandbox smoke
- 已把下一步从“继续写 read-model 切口说明”推进到“handoff receipt / alert cluster 的对应代码 cut”

## 实现摘要

这一阶段最重要的变化，是 `TrackedTaskState` 第一次正式拥有了 follow-up task detail 字段，而且这些字段不会破坏现有 file / SQLite 持久化和旧任务回读。这使得 `tasks/intake` 与 `tasks/{task_id}` 开始具备结构化 follow-up 入口，而不是只剩 evidence 文本。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `bash scripts/smoke_v0_2_0_sandbox.sh`

## 阶段收口结论

FA 现在已经不仅知道 `follow_up_items` 应该进入任务详情，也已经把最小 schema cut 真实接进了 API 和持久化主链。下一步最合理的动作，是按同样方法继续把 `handoff_receipt` 和 `alert_cluster_drafts` 推进到代码实现，并开始为单场景非空 follow-up draft 做受控填充。
