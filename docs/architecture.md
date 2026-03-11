# FA Architecture

## 1. 目标

FA 的目标是成为制造企业的 Agentic coordination layer，让以下对象在统一规则下协同：

- 企业系统：ERP、MES、CMMS、QMS、WMS、SCADA
- 人员角色：操作员、班组长、质量工程师、设备工程师、厂长
- 现场资产：PLC、机器人、CNC、视觉站、传感器
- AI 能力：规划、诊断、调度、审批辅助、异常处置

首版不是直接追求“自治工厂”，而是先构建一套可治理的工作编排基础设施。

## 2. 设计原则

### 2.1 Safety first

任何可能影响安全、质量放行、设备动作、产线节拍、成本责任归属的动作，都必须支持显式审批和全量审计。

### 2.2 Deterministic wrapper over probabilistic reasoning

LLM/Agent 负责推理、归纳、建议与任务拆解；真正的写操作、设备控制、业务状态变更，必须经过确定性业务规则与 connector policy 包裹。

### 2.3 Human role clarity

AI 不能模糊责任边界。每一步工作流都必须能回答：

- 谁提出任务
- 谁批准
- 哪个 agent 参与
- 哪个系统被访问
- 最终谁对结果负责

### 2.4 Reference before autonomy

第一阶段优先做“辅助执行 + 明确审批”，不是“黑盒自动化”。

## 3. Agentic Pattern 选型

本项目第一阶段采用以下模式组合，依据来自 Google Cloud 的 Agentic AI design pattern 选择框架。

### Single-agent

适用：

- 低风险、单系统、单角色的狭义任务
- 班次总结、异常摘要、工单草稿生成

不适用：

- 涉及设备动作
- 涉及财务、质量放行、跨部门流程

### Coordinator

适用：

- 跨 ERP / MES / CMMS / SCADA 的多系统任务
- 同时涉及班组长、设备工程师、质量工程师的协同工作

这是本项目的主编排模式。

### ReAct loop

适用：

- 故障诊断
- 异常根因分析
- 订单例外处理
- 工艺偏差分析

要求：

- 必须输出证据来源
- 必须输出建议动作与置信度
- 不允许跳过审批直接执行高风险动作

### Human-in-the-loop

适用：

- 安全相关动作
- 质量放行/冻结
- 高成本影响决策
- 设备参数调整
- 关键流程变更

要求：

- 审批人必须明确
- 审批意见必须落库
- 被拒绝后必须支持回退或重规划

### Deterministic workflow + custom business logic

适用：

- 所有外部系统写操作
- 所有现场设备相关动作
- 所有企业级 SOP 和审批策略

这是将 AI 建议变成企业可执行流程的关键。

## 4. 系统分层

### 4.1 Experience and API layer

职责：

- 对前端、移动端、管理员端、集成方暴露统一 API
- 提供 health、planning、approval、execution、audit 接口

当前实现：

- `apps/fa-server`

### 4.2 Orchestration layer

职责：

- 选择任务对应的 agentic pattern
- 生成执行计划
- 驱动审批与执行状态机
- 管理 agent 间委派

当前实现：

- `crates/fa-core`

### 4.3 Domain and connector layer

职责：

- 定义企业组织、人员、设备、任务与集成语义
- 屏蔽不同工厂系统的差异
- 提供安全边界

当前实现：

- `crates/fa-domain`

### 4.4 Governance and observability layer

职责：

- 审计日志
- Trace 与 metrics
- 版本治理
- 发布追踪
- 风险与变更管理

当前状态：

- 文档和 CI 基线已建立
- 内存、文件和 SQLite 三种本地持久化基线已建立
- 首条 pilot workflow 已完成候选比较与规格定义基线
- 任务级 evidence snapshot 已进入 API 与持久化主链

## 5. 初始数据流

1. 人员或系统发起任务请求。
2. API 层完成入参校验并补齐上下文。
3. Orchestrator 根据风险、优先级、系统跨度、设备关联度选择模式。
4. 如需诊断，进入 ReAct loop 获取证据与建议。
5. 如需写操作或设备影响，进入 deterministic workflow 与 policy checks。
6. 如需审批，则挂入 human-in-the-loop 节点。
7. 执行结束后记录 outcome、evidence、audit trail。

## 6. 非功能要求基线

### 6.1 安全

- 所有高风险工作流必须支持审批
- 所有外部写操作必须支持 policy gating
- 不允许裸奔调用设备执行器

### 6.2 可观测性

- 每个任务必须具备 trace id / request id
- 关键节点必须可回溯

### 6.3 可扩展性

- connector 必须 trait 化
- agent 选择逻辑与领域模型解耦

### 6.4 可交付性

- 每个里程碑必须对应测试方案与验收标准
- 发布必须与 changelog 和 tag 对应

## 7. 下一阶段架构落点

- 引入 `connector` abstraction 与只读适配器
- 增加任务状态机与审批状态机
- 建立审计存储
- 接入 LLM provider abstraction
- 让首条 pilot workflow 与当前 API、connector 和 evidence 边界逐项对齐
