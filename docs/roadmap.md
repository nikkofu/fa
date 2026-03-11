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
