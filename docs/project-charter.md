# FA Project Charter

## 1. 项目名称

FA Manufacturing Agentic Platform

## 2. 项目起始日期

2026-03-11

## 3. 项目使命

为生产制造型企业建立一个可治理的 Agentic AI 协同平台，使 AI 能在企业、组织、人员、设备和现有业务系统之间参与日常工作，但始终处于明确的安全、审批、审计和版本治理框架内。

## 4. 业务问题

典型制造企业同时面临：

- 生产异常处理依赖人工经验，响应慢且难复制
- ERP、MES、CMMS、QMS 与现场数据链路割裂
- AI 试点停留在问答层，无法进入真实流程
- 涉及质量、安全、设备的动作无法放心交给黑盒模型

## 5. 项目目标

### 5.1 90 天目标

- 建立 Rust 核心平台与可运行 API
- 完成制造领域模型、任务编排和审批门控基线
- 打通至少 2 个企业系统的只读集成原型
- 定义 1 条可试运行的制造场景闭环

### 5.2 成功标准

- 可演示从任务发起到执行计划输出的完整链路
- 关键高风险任务具备明确审批策略
- 代码、文档、版本发布和测试流程在 GitHub 仓库中可追踪
- 能支持试运行前的 UAT 与 SOP 对齐

## 6. 范围

### In scope

- Rust workspace、服务层、编排层、领域层
- 制造任务编排和模式选择逻辑
- 审批与审计设计
- 版本发布和变更说明
- 初始连接器抽象和试运行设计

### Out of scope for phase 1

- 直接控制生产设备的闭环自动执行
- 全量 ERP/MES/CMMS 深度写入
- 多工厂统一主数据平台
- 通用型大模型训练平台

## 7. 关键角色

- Project sponsor: 业务发起人与预算负责人
- Product owner: 生产制造业务负责人
- Solution architect / Tech lead: 平台架构、技术路线、关键实现决策
- Delivery lead / PMP: 里程碑、风险、范围、资源与验收管理
- QA lead: 测试策略、UAT、回归和上线检查
- Pilot plant owner: 试运行工厂负责人

当前仓库默认将技术负责人、0-1 架构与平台落地职责收敛到同一交付主线。

## 8. 里程碑

### M0: Foundation

时间：2026-03-11 至 2026-03-20

交付：

- 仓库初始化
- Rust workspace
- 初始架构文档、ADR、CI、release 流程

### M1: Core orchestration prototype

时间：2026-03-23 至 2026-04-10

交付：

- connector trait
- 审批与执行状态机
- 读链路系统接入样例
- 试运行场景定义

### M2: Pilot-ready workflow

时间：2026-04-13 至 2026-05-29

交付：

- 至少 1 条端到端 pilot workflow
- UAT 用例
- 可观测性和审计存储

### M3: Trial run and release baseline

时间：2026-06-01 至 2026-07-31

交付：

- 试运行
- 缺陷修复
- 发布节奏建立
- 运维与交付手册

## 9. 风险登记册

### R1. 业务范围膨胀

风险：一开始就试图覆盖全部制造流程。

应对：严格围绕 1 条高价值 pilot workflow 推进。

### R2. AI 自动化越过治理边界

风险：为了追求“智能”，跳过审批和 SOP。

应对：所有写操作和设备动作必须走 deterministic policy gate。

### R3. 集成复杂度低估

风险：ERP/MES/CMMS 接口条件、主数据质量、权限模型不一致。

应对：先做 connector abstraction 和 read-only proof-of-concept。

### R4. 试运行验收标准不清

风险：开发完成但无法上线试运行。

应对：在 M1 阶段即定义 UAT、KPI、回退和责任矩阵。

## 10. 交付节奏

- 每周交付一次仓库可运行版本
- 每个里程碑输出文档、代码、测试与风险更新
- 所有重要变化必须进入 changelog 和 release note

## 11. 团队工作流基线

项目团队的详细工作流、角色协作方式、RACI、生命周期、变更与风险控制机制，统一定义在：

- [governance/README.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/README.md)
- [governance/team-operating-model.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/team-operating-model.md)
- [governance/delivery-lifecycle.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/delivery-lifecycle.md)
- [governance/governance-controls.md](/Users/admin/Documents/WORK/ai/fa/docs/governance/governance-controls.md)
