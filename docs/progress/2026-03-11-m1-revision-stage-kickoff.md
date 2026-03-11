# M1 Revision Stage Kickoff

## 日期

2026-03-11

## 同步目的

在进入 `M1-W06 Revision Resubmission Loop` 前，明确当前版本、GitHub 基线和本阶段要补齐的业务闭环，避免让“驳回后修订”停留在半成品状态。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`rejected -> planned -> resubmitted approval`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`9088157`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- `TaskRepository` abstraction
- `InMemoryTaskRepository`
- `WorkOrchestrator` repository 注入
- repository 单元测试与运行态 smoke

## 本阶段目标

当前系统已经能在审批驳回时把任务退回 `planned`，但还缺少从修订态重新申请审批的显式能力。这会让“revision loop”在真实业务里卡住。

本阶段目标：

1. 增加 rejected work 的 resubmit 能力。
2. 保持现有生命周期和 repository abstraction 行为稳定。
3. 为后续“修订后重提”业务流程建立可验证的最小闭环。

## 本阶段交付边界

本阶段计划交付：

- resubmit request model
- orchestrator resubmit action
- HTTP endpoint
- service-level revision loop tests
- 文档同步

本阶段暂不交付：

- 多轮修订版本 diff 管理
- 修订内容字段模型
- 审批 SLA / 超时处理

## 风险与注意事项

- 如果只增加 endpoint，不验证完整拒绝后重提路径，闭环仍然不可信。
- 如果引入过多修订元数据，会拖慢 `M1` 节奏。
- 必须保留 audit 与 correlation id 的一致性。

## 进入本阶段的理由

工业现场真正有价值的流程，不是“第一次就批准”，而是“被驳回后还能在治理框架内修订并重提”。补上这个环节，平台才更接近真实制造组织的工作方式。

## 当前推进结果

本阶段已经完成：

- `ResubmitTaskRequest`
- orchestrator `resubmit_task(...)`
- `POST /api/v1/tasks/{task_id}/resubmit`
- 服务层 revision loop 集成测试
- 驳回后非法执行拦截测试
- 文档同步

## 当前验证结果

已完成的质量验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- 在 `127.0.0.1:8000` 下完成 `intake -> reject -> resubmit -> approve` HTTP smoke test

当前结论：

- `approve(false) -> resubmit -> approve(true)` 已可用
- rejected work 不再需要重建任务才能重新走审批
- 修订闭环已从设计概念进入可验证接口能力
