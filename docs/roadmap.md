# FA Roadmap

## 2026-03-11 Baseline

版本目标：`v0.1.0`

已建立：

- Rust workspace
- 领域模型
- 编排器与模式选择
- API skeleton
- 文档、CI、Release 基线

## v0.2.0

目标日期：2026-03-31

当前状态：`In Progress`

当前进行中的执行切片：`M1-W07 Local Persistence Baseline`

已完成的子项：

- 任务状态机
- 审批状态机
- `connector` trait
- `audit` trait
- request / trace correlation id
- mock `MES` / `CMMS` read-only connector
- `intake / get / approve / execute / complete / fail` 生命周期主路径
- 服务层生命周期集成测试
- task repository abstraction
- rejected task resubmission loop
- local persistence baseline for task and audit storage

计划内容：

- 任务状态机
- 审批状态机
- `connector` trait
- `audit` trait
- request / trace correlation id
- 开始接入只读 MES / CMMS mock connector

验收条件：

- 任务不再只有规划结果，而具备生命周期
- connector 可替换，不与具体供应商绑定
- 生命周期 API 主路径可通过服务层测试验证
- task state storage 可通过 repository abstraction 替换而不改 API 主路径
- rejected work 可重新发起审批而不必重建任务
- 本地文件持久化可支撑重启后的任务与审计回读

执行计划：

- [planning/m1-execution-plan.md](/Users/admin/Documents/WORK/ai/fa/docs/planning/m1-execution-plan.md)

## v0.3.0

目标日期：2026-04-18

计划内容：

- LLM provider abstraction
- evidence store abstraction
- prompt / tool / policy boundary 分层
- 高风险任务审批链

验收条件：

- 能基于外部知识与业务上下文完成一次带证据的诊断规划

## v0.4.0

目标日期：2026-05-15

计划内容：

- Pilot workflow 端到端打通
- 审计日志与回放视图
- UAT 测试脚本
- 部署清单与试运行指南

验收条件：

- 支持至少 1 条制造试运行场景闭环

## v1.0.0

前提：

- 至少完成一次受控试运行
- 发布、回退、审计、权限与 KPI 管理完整
- 已验证真实业务价值和运行稳定性
