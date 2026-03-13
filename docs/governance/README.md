# Governance Docs

本目录定义 FA 项目的团队协作与项目治理操作体系。它补足了项目章程、架构和发布流程之间的空白，回答三个问题：

1. 团队里谁负责什么。
2. 一个需求或问题如何从提出走到发布与试运行。
3. 项目如何做范围、风险、质量、变更与节奏控制。

## 适用范围

本目录适用于以下活动：

- 产品需求发现与范围确认
- Rust 平台研发、代码评审与测试
- AI/Agent 能力引入的安全与审批治理
- 试运行准备、发布、回退与复盘
- PMP 维度的进度、风险、依赖与变更管理

## 文档清单

- [team-operating-model.md](team-operating-model.md)
  定义团队角色、职责、RACI、会议节奏、沟通规则和协作约定。
- [delivery-lifecycle.md](delivery-lifecycle.md)
  定义从需求进入到发布关闭的端到端工作流、状态流转、阶段出口和 DoR/DoD。
- [governance-controls.md](governance-controls.md)
  定义范围、进度、质量、风险、变更、配置、发布和状态汇报控制机制。

## 与现有文档的关系

- [project-charter.md](../project-charter.md)
  定义项目使命、目标、里程碑和高层角色。
- [roadmap.md](../roadmap.md)
  定义版本目标和阶段性产出。
- [architecture.md](../architecture.md)
  定义技术架构与模式选择。
- [release-process.md](../release-process.md)
  定义版本发布动作与检查点。

治理文档解决的是“怎么组织团队把这些目标落地”的问题。

## 执行原则

- 仓库是项目的单一事实源。
- 没有 issue 编号的工作不进入开发。
- 没有验收标准的需求不进入排期。
- 没有测试证据和风险说明的变更不进入发布候选。
- 涉及设备、质量、安全、审批、业务写入的能力必须有明确责任人和回退策略。
