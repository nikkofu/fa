# M1 Lifecycle And Abstractions Design

## 1. 目的

本设计文档固化 `M1-W01` 到 `M1-W04` 的最小设计结论，为后续编码提供一致边界：

- 任务生命周期模型
- 审批生命周期模型
- connector read-only 抽象
- audit event 最小模型

## 2. 任务生命周期

### 2.1 状态

`TaskStatus` 当前采用最小主路径：

- `draft`
- `planned`
- `awaiting_approval`
- `approved`
- `executing`
- `completed`
- `failed`

### 2.2 设计原则

- 只表达当前 `v0.2.0` 需要的主状态流，不提前引入过多分支状态。
- 所有状态转换必须显式校验，不允许随意赋值跳跃。
- 任务记录保留原始 `TaskRequest` 和 `ExecutionPlan`，为后续审计和回放提供基础。

### 2.3 允许的状态迁移

| From | To |
| --- | --- |
| `draft` | `planned` |
| `planned` | `awaiting_approval`, `approved`, `failed` |
| `awaiting_approval` | `planned`, `approved`, `failed` |
| `approved` | `executing`, `failed` |
| `executing` | `completed`, `failed` |

设计解释：

- `awaiting_approval -> planned` 表示审批驳回后返回修订。
- `failed` 目前作为终态，后续如需重试，使用新任务记录或新增重开策略。

## 3. 审批生命周期

### 3.1 状态

`ApprovalStatus` 当前采用：

- `pending`
- `approved`
- `rejected`
- `expired`

### 3.2 设计原则

- `ApprovalRecord` 仅在 `ApprovalPolicy != auto` 时创建。
- 审批记录必须保留请求人、审批角色、审批人、意见、请求时间和决策时间。

### 3.3 允许的状态迁移

| From | To |
| --- | --- |
| `pending` | `approved`, `rejected`, `expired` |

终态不可再次流转。

## 4. Connector 抽象

### 4.1 目标

在 `M1` 阶段，connector 只解决一个问题：

统一表示从外部企业系统读取上下文，而不暴露写操作。

### 4.2 边界

trait:

- `Connector`

核心类型：

- `ConnectorKind`
- `ConnectorAccess`
- `ConnectorSubject`
- `ConnectorRecordKind`
- `ConnectorReadRequest`
- `ConnectorReadResult`

### 4.3 设计原则

- 默认 `read-only`
- 不绑定具体供应商 SDK
- 返回结构化记录，而不是裸 JSON blob
- 每次读取应支持关联 `correlation_id`

## 5. Audit 抽象

### 5.1 目标

为任务状态变化、审批事件和 connector 读取建立最小可追溯模型。

### 5.2 核心类型

- `AuditEventKind`
- `AuditActor`
- `AuditEvent`
- `AuditSink`

### 5.3 首批事件

- `task_created`
- `task_planned`
- `task_status_changed`
- `approval_requested`
- `approval_approved`
- `approval_rejected`
- `approval_expired`
- `connector_read`

### 5.4 设计原则

- audit 接口先抽象，持久化实现后续单独演进
- event 必须允许挂靠 `task_id`、`approval_id`、`correlation_id`
- actor 必须区分 human / agent / system

## 6. 当前实现落点

- 领域生命周期模型：`crates/fa-domain/src/lifecycle.rs`
- connector 抽象：`crates/fa-core/src/connectors.rs`
- audit 抽象：`crates/fa-core/src/audit.rs`
- 内存任务存储与动作编排：`crates/fa-core/src/orchestrator.rs`
- mock connector 实现：`MockMesConnector`、`MockCmmsConnector`
- audit 实现：`InMemoryAuditSink`
- 生命周期 API：
  - `POST /api/v1/tasks/intake`
  - `GET /api/v1/tasks/{task_id}`
  - `POST /api/v1/tasks/{task_id}/approve`
  - `POST /api/v1/tasks/{task_id}/execute`
  - `GET /api/v1/audit/events`

## 7. 下一步

基于本设计，下一步实现顺序应为：

1. 增加任务完成、失败和回退动作
2. 为 API 层补完整集成测试
3. 将 in-memory task store 演进为可替换 repository
4. 为 connector 和 audit 增加更完整的契约测试
