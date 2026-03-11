# M1 Evidence Snapshot Baseline Kickoff

## 日期

2026-03-12

## 同步目的

在首条 pilot workflow 规格已经冻结后，进入一个更贴近实现的对齐阶段：把 workflow 中的“证据清单”从概念说明推进为真实系统对象，使证据能够进入任务状态、API 输出和持久化存储。

## 当前版本状态

- 已发布版本：`v0.1.0`
- 当前开发目标：`v0.2.0`
- 当前里程碑：`M1 - Core orchestration prototype`
- 当前阶段主题：`structured task evidence snapshots`

## GitHub 仓库状态

- GitHub 仓库：`https://github.com/nikkofu/fa`
- 当前远端同步 commit：`6a6317c`
- 当前本地分支状态：与 `origin/main` 同步

## 上一阶段完成基线

上一阶段已完成并推送：

- 首条 pilot workflow 候选比较
- 第一版 pilot workflow spec
- workflow 的业务边界、审批角色、回退策略定义

## 本阶段目标

本阶段目标：

1. 为任务引入结构化 evidence snapshot。
2. 让 `context_reads` 生成可回读的证据对象，而不只是原始 connector 返回。
3. 让 evidence 跟随内存、文件、SQLite 三种模式一起持久化。
4. 暴露任务级 evidence 查询接口，并补齐测试与文档示例。

## 本阶段交付边界

本阶段计划交付：

- `TaskEvidence` 最小模型
- evidence 生成逻辑
- task evidence 查询 API
- 相关单测、集成测试
- README / roadmap / changelog / progress / journal 同步

本阶段暂不交付：

- 独立 evidence store
- evidence 图谱或向量检索
- 跨任务 evidence 关联分析

## 风险与注意事项

- evidence 模型必须保持轻量，不能在 `v0.2.0` 前引入过重的数据层设计
- 不能只为了“有 evidence 字段”而复制一份原始 connector 数据；必须表达结构化摘要价值
- 现有 API 与存储兼容性不能被破坏

## 进入本阶段的理由

pilot workflow 如果没有结构化 evidence，只能停留在“能读取上下文，但不能把证据作为正式输出”的状态。这会直接削弱后续审批、回放和试运行说服力，因此 evidence snapshot 是当前最应该补上的一层。

## 本阶段完成结果

- 已交付 `TaskEvidence` 最小模型
- 任务 `intake` 和 `get` 返回中已包含 `evidence`
- 已交付 `GET /api/v1/tasks/{task_id}/evidence`
- evidence 已随任务状态一起持久化到内存、文件和 SQLite 三种模式

## 实现摘要

本阶段没有另起一个重量级 evidence store，而是先把 evidence 作为任务级 snapshot 引入当前主链。这让平台在不打断 `v0.2.0` 节奏的前提下，把“证据驱动”落进了真实 API 和真实持久化层。

## 验证记录

已完成验证：

- `cargo fmt --all`
- `cargo test --workspace`

验证覆盖点：

- evidence 从 connector 读取结果中生成
- task intake 会生成 evidence 快照
- task evidence 可通过 orchestrator 和 HTTP API 查询
- 文件模式和 SQLite 模式下的任务状态持久化继续成立

## 阶段收口结论

evidence snapshot 基线已经成立，pilot workflow 的“证据清单”不再只是文档描述，而是实际系统输出。下一阶段应把重点转向 `v0.2.0` 的测试/发布清单和更明确的责任矩阵表达。
