# M1 Lifecycle And Abstractions Design

## 1. 目的

本设计文档固化当前 `M1` 阶段的最小设计结论，为后续编码提供一致边界：

- 任务生命周期模型
- 审批生命周期模型
- connector read-only 抽象
- audit event 最小模型
- task repository abstraction
- audit replay query baseline
- task evidence snapshot baseline
- workflow governance matrix baseline

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
- `AuditEventQuery`
- `AuditStore`

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
- audit 查询必须至少支持 `task_id`、`correlation_id`、`kind` 这些运行主键
- 当前持久化实现已覆盖 `InMemoryAuditSink`、`FileAuditStore` 和 `SqliteAuditStore`

## 6. Task Repository 抽象

### 6.1 目标

把任务状态存储从 `WorkOrchestrator` 内部实现细节，提升为可替换边界。

### 6.2 边界

trait:

- `TaskRepository`

当前实现：

- `InMemoryTaskRepository`
- `FileTaskRepository`
- `SqliteTaskRepository`

### 6.3 设计原则

- 编排器不直接绑定某种具体存储结构
- 当前阶段优先稳定接口，不提前引入数据库特定能力
- 保持当前生命周期 API 行为不变
- 为后续持久化、回放和 optimistic locking 留出演进空间

## 7. Task Evidence Snapshot

### 7.1 目标

把 connector 的原始读取结果进一步沉淀为任务级 evidence 快照，使“证据清单”成为任务正式输出的一部分，而不是只存在于底层上下文读取里。

### 7.2 核心类型

- `TaskEvidence`
- `TrackedTaskState.evidence`

### 7.3 设计原则

- evidence 先作为轻量 snapshot 存在，不单独引入复杂存储层
- evidence 必须可序列化、可持久化、可通过任务接口回读
- evidence 要保留来源、记录类型、摘要和原始 payload
- evidence 由 connector read 结果生成，但不等同于简单复制原始读取对象

## 8. Workflow Governance Baseline

### 8.1 目标

把首条 pilot workflow 的治理要求沉淀为任务计划中的正式对象，使责任矩阵、审批策略和 fallback actions 能被任务接口直接回读。

### 8.2 核心类型

- `WorkflowGovernance`
- `ResponsibilityAssignment`
- `GovernanceParticipation`
- `ApprovalStrategy`

### 8.3 设计原则

- governance 先绑定在 `ExecutionPlan` 上，而不是额外引入复杂配置中心
- responsibility matrix 要能表达角色与参与方式，而不只是自然语言描述
- approval strategy 要能表达 required role、决策范围和 escalation role
- governance 必须进入 API、任务存储和 smoke path，而不是停留在设计文档

## 9. 持久化运行时选择

### 7.1 当前运行时顺序

`fa-server` 当前按以下顺序注入持久化后端：

1. `FA_SQLITE_DB_PATH` -> `SqliteTaskRepository` + `SqliteAuditStore`
2. `FA_DATA_DIR` -> `FileTaskRepository` + `FileAuditStore`
3. 未设置时 -> `InMemoryTaskRepository` + `InMemoryAuditSink`

### 7.2 设计原则

- API 层和编排器不感知底层存储类型
- SQLite 基线用于本地结构化耐久存储与重启回读验证
- 不把当前 SQLite 基线误写成最终企业数据库方案
- 保持三种模式并存，避免打断当前开发、测试和演示流

## 10. 当前实现落点

- 领域生命周期模型：`crates/fa-domain/src/lifecycle.rs`
- connector 抽象：`crates/fa-core/src/connectors.rs`
- audit 抽象：`crates/fa-core/src/audit.rs`
- evidence 抽象：`crates/fa-core/src/evidence.rs`
- governance 抽象：`crates/fa-domain/src/workflow.rs`
- task repository abstraction：`crates/fa-core/src/repository.rs`
- 生命周期动作编排：`crates/fa-core/src/orchestrator.rs`
- mock connector 实现：`MockMesConnector`、`MockCmmsConnector`
- audit 实现：`InMemoryAuditSink`、`FileAuditStore`、`SqliteAuditStore`
- repository 实现：`InMemoryTaskRepository`、`FileTaskRepository`、`SqliteTaskRepository`
- task state 输出：`correlation_id`、`planned_task`、`context_reads`、`evidence`
- task plan 输出：`patterns`、`approval_policy`、`governance`、`steps`
- 生命周期 API：
  - `POST /api/v1/tasks/intake`
  - `GET /api/v1/tasks/{task_id}`
  - `GET /api/v1/tasks/{task_id}/evidence`
  - `GET /api/v1/tasks/{task_id}/governance`
  - `GET /api/v1/tasks/{task_id}/audit-events`
  - `POST /api/v1/tasks/{task_id}/approve`
  - `POST /api/v1/tasks/{task_id}/resubmit`
  - `POST /api/v1/tasks/{task_id}/execute`
  - `POST /api/v1/tasks/{task_id}/complete`
  - `POST /api/v1/tasks/{task_id}/fail`
  - `GET /api/v1/audit/events`

## 11. 下一步

基于本设计，下一步实现顺序应为：

1. 准备 `v0.2.0` 版本号、tag 和最终 release note
2. 为 approval action 增加更严格的审批角色校验与策略约束
3. 评估 SQLite 向更强数据库后端的迁移边界
4. 扩展 evidence、审批 SLA 与异常路径
