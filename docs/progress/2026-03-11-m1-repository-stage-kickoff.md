# M1 Repository Stage Kickoff

## 日期

2026-03-11

## 同步目的

在进入 `M1-W05 Repository Abstraction` 前，明确当前版本、远端基线、阶段范围和风险，保证后续实现不是在模糊状态下推进。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`replaceable task repository abstraction`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`f83c5af`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- `intake / get / approve / execute / complete / fail` 生命周期闭环
- 服务层生命周期集成测试
- 审计回看接口
- 阶段进展与工作心得文档基线

## 本阶段目标

本阶段要解决的核心问题，不是“再加一个接口”，而是把当前写死在编排器内部的内存任务存储抽出来，形成可替换的 repository 边界。

本阶段目标：

1. 引入 `task repository` 抽象。
2. 保留当前 API 行为不变。
3. 保留当前测试与 smoke path 行为不变。
4. 为后续持久化实现留出注入点。

## 本阶段交付边界

本阶段计划交付：

- repository trait
- in-memory repository 实现
- `WorkOrchestrator` repository 注入能力
- 不破坏现有 HTTP API 的回归测试
- 文档同步

本阶段暂不交付：

- SQLite / Postgres 持久化实现
- 审计事件持久化
- repository optimistic locking

## 风险与注意事项

- 如果 repository 接口设计得过于贴近当前内存实现，后续切持久化仍会返工。
- 如果 repository 接口设计得过于理想化，又会拖慢当前 `M1` 节奏。
- 必须避免因为抽象重构而破坏生命周期与审计主路径。

## 进入本阶段的理由

当前生命周期主链已经跑通。如果继续堆功能，而不先抽出任务状态存储边界，后续任何持久化、回放和试运行准备都会被内部结构绑死。现在做 repository abstraction，是为了让系统从“能演示”迈向“能演进”。

## 当前推进结果

本阶段已经完成：

- `TaskRepository` trait
- `InMemoryTaskRepository` 实现
- `WorkOrchestrator` repository 注入能力
- repository 单元测试
- repository 注入验证测试
- 现有 HTTP 层测试保持通过

## 当前验证结果

已完成的质量验证：

- `cargo fmt --all`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- 在 `127.0.0.1:8000` 下完成实际 HTTP smoke test

当前结论：

- 生命周期 API 行为保持稳定
- task state storage 已从编排器内部实现细节提升为可替换边界
- 下一步可以在不重写 API 主路径的前提下引入持久化实现
