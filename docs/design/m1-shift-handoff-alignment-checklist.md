# M1 Shift Handoff Workflow Alignment Checklist

## 1. 文档目的

本文件把 `班次交接摘要与待办提取` 从 workflow specification baseline 推进到实现对齐层，重点回答 5 个问题：

1. 当前平台已经有哪些能力可以直接复用到交接场景。
2. `MES / shift log / incident log / task history / CMMS` 分别缺什么 connector 基线。
3. 当前 `TaskEvidence`、`follow-up / SLA` 模型和 governance 能否承接交接语义。
4. 当前 API 是否足以支撑第一轮交接场景演示。
5. 后续应按什么顺序进入实现，而不是把交接场景继续留在摘要概念层。

本文件不是实现说明，也不是“交接场景已经打通”的声明。它是后续 connector、evidence、follow-up、governance 和 API 演进的对齐清单。

## 2. 对齐原则

交接场景进入实现前，继续坚持以下原则：

- 先做只读摘要、待办提取和风险提醒，不做自动确认与自动重分配。
- 先把 follow-up / owner / due window 作为 read-only draft 输出，再考虑接收确认。
- 先用 mock `shift log / incident log` 验证主链，再讨论真实系统接入。
- 先让 API 能回放“班次事件 -> 摘要 -> follow-up -> 风险升级候选”链路，再扩展交接专属对象。

## 3. 当前平台基线快照

| 对象 | 当前状态 | 对交接场景的意义 |
| --- | --- | --- |
| `TaskRequest` | 已支持通用任务字段、角色、风险、集成目标、期望结果 | 能发起交接任务，但没有 `shift_id / handoff_window / receiving_shift` 等结构化字段 |
| `TrackedTaskState` | `tasks/intake` 返回 `planned_task / context_reads / evidence / correlation_id` | 交接场景可以复用同一条任务主链 |
| `TaskEvidence` | 已支持 `connector / record_kind / source_ref / observed_at / summary / payload` | 可以承接交接线索，但不适合表达时间线、遗留事项和接收状态 |
| `WorkflowGovernance` | 已支持责任矩阵、审批策略和 fallback actions | 可复用责任矩阵，但交接场景更偏确认与接收，不偏审批 |
| `follow-up / SLA` | 已有独立对齐清单，但尚无正式对象 | 交接场景是这层模型最直接的落地入口之一 |
| connector registry | 默认只注册 mock `MES / CMMS` | `shift log / incident log` 当前既没有 target 语义，也没有默认 connector |

## 4. Connector 对齐清单

### 4.1 最小必需连接器

| 系统 | 交接 workflow 最小必需记录 | 当前状态 | 建议 |
| --- | --- | --- | --- |
| `MES` | 班次工单、产量、停机、基础异常、设备状态摘要 | 已有 mock `MES`，但当前 payload 偏设备异常场景，只返回 `TaskContext / EquipmentTelemetry` | Phase A 扩展 mock `MES`，补 shift-level summary、line status、exception count |
| `Shift Log` | 交接备注、重点事项、人工补充、下一班提醒 | 当前没有一等 integration target，可先用 `Custom("shift_log")` 表达 | Phase A 先补 mock `shift log` read-only connector |
| `Incident Log` | 未关闭异常、阻塞项、临时处置说明 | 当前没有一等 integration target，可先用 `Custom("incident_log")` 表达 | Phase A 与 shift log 一起补 mock baseline |
| `Task History / Audit` | 已跟踪任务、审批结果、未完成 follow-up 线索 | 任务查询与任务级审计接口已存在，但没有专门的跨任务聚合读链路 | Phase B 先以查询聚合适配或 mock projection 方式承接 |
| `CMMS` | 未关闭设备事项、维护建议、维修遗留风险 | 已有 mock `CMMS`，但当前返回的是设备异常上下文 | Phase A 可直接复用，再按交接语义补充未关闭事项表达 |

### 4.2 当前代码层面的关键差距

当前与交接连接器最相关的差距主要有 3 点：

1. 默认 `ConnectorRegistry` 只注册了 mock `MES / CMMS`。
2. `requested_record_kinds()` 当前只为 `MES` 和 `CMMS` 生成读取记录，`Custom("shift_log") / Custom("incident_log")` 目标不会返回有效 records。
3. `ConnectorRecordKind` 当前只有 `TaskContext / EquipmentTelemetry / MaintenanceHistory / WorkOrderContext / QualityContext / Custom(_)`，还没有适合班次交接的时间线或备注语义。

这意味着交接场景虽然已经有 spec，但在运行主链上仍然缺少真正可读的班次记录和交接原始输入。

### 4.3 Connector 实施顺序建议

1. mock `shift log` read-only baseline
2. mock `incident log` read-only baseline
3. mock `MES` shift summary payload 补强
4. 任务 / 审计聚合投影视图
5. `CMMS` 未关闭事项的交接语义补强

## 5. Evidence 与 Follow-up 对齐清单

### 5.1 交接场景至少要表达的对象

| 对象 | 为什么必须有 | 当前是否能表达 | 差距 | 建议 |
| --- | --- | --- | --- | --- |
| `shift_id / handoff_window` | 这是交接任务的基础上下文 | 只能放在描述文本里 | 无一等字段，难以检索与回放 | Phase A 先放入 payload；Phase B 再考虑任务元数据字段 |
| `key events timeline` | 交接不是单点事实，而是一段时间窗口内的事件串 | 当前 `TaskEvidence` 是离散 snapshot | 时间顺序与事件归并表达不足 | Phase A 先用 payload 时间线；Phase B 再补 timeline-style evidence |
| `unresolved issue list` | 交接的核心是遗留问题 | 只能放在自由文本或 evidence payload | 无结构化 item 列表 | 依赖 follow-up model Phase A |
| `recommended owner / receiving role` | 交接必须清楚下一步谁接手 | 只能写在摘要文本里 | 不能表达推荐与接受之间的差异 | 依赖 follow-up owner model |
| `due window` | “下一班前”“两小时内”这类时限是交接关键 | 当前无字段 | 时限无法排序、升级和查询 | 依赖 follow-up / SLA model |
| `handoff confirmation / receipt` | 交接不是生成摘要就结束，还要确认被接收 | 当前无对象 | 无法表达“已发出但未接收”的风险 | Phase B 增加 receipt / acknowledgement object |
| `blocking reason` | 许多交接问题不是未开始，而是被阻塞 | 当前无子项级状态 | 无法区分普通遗留与阻塞遗留 | 依赖 follow-up model |

### 5.2 当前 `TaskEvidence` 能做什么

当前 `TaskEvidence` 的优点是：

- 能快速承接 `MES / CMMS / custom log` 读取结果
- 具备来源、时间戳和 connector 维度
- 可以通过 `tasks/intake`、`tasks/{task_id}`、`tasks/{task_id}/evidence` 一致输出

当前 `TaskEvidence` 的局限是：

- 无法稳定表达班次事件时间线
- 无法表达遗留事项与 follow-up item 的差异
- 无法表达交接摘要是否被确认、被谁接收
- 无法表达 due window、blocked reason、receipt status

### 5.3 与 follow-up / SLA 模型的关系

交接场景是 follow-up / SLA 模型最直接的落地场景之一：

- 每条遗留事项都应映射为 `follow-up item`
- `recommended owner / accepted owner / due window / blocked status` 都应是正式对象
- 交接摘要中的“下一班重点”不应长期停留在自由文本里

因此交接对齐清单不能独立于 follow-up / SLA checklist 看待，两者是同一条实现链路的前后层。

## 6. Governance 对齐清单

### 6.1 角色与确认要求

| 治理项 | 当前状态 | 交接场景差距 | 建议 |
| --- | --- | --- | --- |
| accountable role | 当前高风险治理偏 `Safety Officer / Plant Manager` 审批路径 | 交接场景更偏 `Production Supervisor` 确认与 `Incoming Shift Supervisor` 接收 | Phase B 为交接场景补 `confirmation / receipt` 语义，而不是硬套审批 |
| consulted roles | 当前可输出 `quality_engineer`、`maintenance_engineer` 等责任矩阵 | 交接场景需要更清晰表达 `Shift Lead / Incoming Shift Supervisor` 的参与边界 | 在交接 governance builder 中固化确认与接收角色 |
| escalation path | 当前 governance 支持 generic fallback 与升级说明 | 交接场景需要表达“未接收”“超时未处理”“高风险遗留未升级” | 依赖 follow-up / SLA policy 输出 |
| forbidden actions | 当前无真实写 connector，因此默认安全 | 交接场景仍需明确禁止自动确认、自动改派和自动关单 | 保持 read-only / draft baseline |
| approval requirement | 当前平台擅长审批门控 | 交接场景不是审批型流程，强行审批会错配 | 先保持 confirmation / acknowledgement 语义，避免伪审批 |

### 6.2 与当前设备异常治理的关键差异

交接场景与当前 pilot 最大差异不是风险低，而是治理目标不同：

- 交接重点是确认、接收、提醒和升级
- 设备异常重点是审批、执行和回退
- 交接需要表达“谁接手”和“什么时候必须跟”
- 而不是默认把所有输出都塞进审批策略

## 7. API 对齐清单

### 7.1 当前已经可复用的接口

| 接口 | 当前是否可用于交接 workflow | 说明 |
| --- | --- | --- |
| `POST /api/v1/tasks/plan` | 可以 | 可先生成交接任务的模式选择与治理草图 |
| `POST /api/v1/tasks/intake` | 可以 | 已能返回 `planned_task / context_reads / evidence / correlation_id` |
| `GET /api/v1/tasks/{task_id}` | 可以 | 可查看任务状态、计划与 evidence |
| `GET /api/v1/tasks/{task_id}/evidence` | 可以 | 适合查看班次线索和交接原始记录 |
| `GET /api/v1/tasks/{task_id}/governance` | 可以 | 可验证交接责任矩阵与 fallback 边界 |
| `GET /api/v1/tasks/{task_id}/audit-events` | 可以 | 可回放交接任务轨迹 |

### 7.2 当前 API 的主要差距

| 能力 | 当前状态 | 建议 |
| --- | --- | --- |
| 交接任务 intake 字段 | 通用字段够发起任务，但没有 `shift_id / handoff_window / receiving_shift` | Phase B 再决定是否扩 request schema；Phase A 先用 payload 承接 |
| follow-up items | 当前无法结构化返回遗留事项列表 | 依赖 follow-up model Phase A |
| receipt / acknowledgement | 当前没有交接已接收状态对象 | Phase B 增加 receipt read model |
| overdue / SLA 查询 | 当前没有交接超时查询能力 | 依赖 follow-up / SLA model 与后续 query |
| cross-task unresolved view | 当前只能单任务查看 evidence 与 audit | Phase C 再补聚合视图 |

### 7.3 当前阶段不必新增的接口

在 connector 和 follow-up 模型未冻结前，不需要立即新增交接专属 endpoint。

更合理的顺序是：

1. 先让交接场景能通过现有任务主链拿到真实 evidence。
2. 再让 follow-up items 和 receipt state 进入任务读模型。
3. 最后再决定是否需要交接专属聚合接口。

## 8. 分阶段实施建议

### Phase A. Mock Shift Log + Evidence Baseline

目标：

- 让交接任务第一次拿到真实可见的班次与交接证据，而不只是人工描述

建议交付：

1. 注册 mock `shift log` / `incident log` read-only connector。
2. 扩展 mock `MES`，补班次摘要与异常计数上下文。
3. 让交接任务通过 `tasks/intake` 和 `tasks/{task_id}/evidence` 展示真实交接线索。
4. 保持所有输出为只读摘要与草稿。

退出标准：

- 交接任务的 evidence 不再只来自人工描述。
- 审计中能看到 `MES / shift log / incident log` 的 connector reads。

### Phase B. Follow-up and Receipt Baseline

目标：

- 让交接摘要从“总结”推进到“有结构化遗留事项和接收状态”

建议交付：

1. 接入 follow-up item read model。
2. 引入 recommended owner / accepted owner / due window。
3. 增加 receipt / acknowledgement object。
4. 让高风险遗留事项具备升级候选标记。

退出标准：

- 交接任务能稳定输出结构化遗留事项，而不是只在摘要中列点。

### Phase C. Cross-shift Aggregation and SLA

目标：

- 让交接场景真正具备“跨班次遗留跟踪”的产品能力

建议交付：

1. 增加 overdue / SLA policy 输出。
2. 支持按班次、owner、状态查看 unresolved follow-up。
3. 支持 receipt overdue 与高风险遗留未升级的提醒。
4. 与 follow-up / SLA query 模型联动。

退出标准：

- 用户能从单次交接摘要走到班次级遗留与超时视图。

## 9. 当前最值得立即推进的实现项

如果只选 3 个最该落地的动作，建议顺序如下：

1. mock `shift log / incident log` read-only baseline
2. 扩展 mock `MES` 的 shift summary evidence
3. 把交接遗留事项接入 follow-up item read model

## 10. 结论

交接场景当前最缺的，不是新的摘要提示词，而是三层更基础的能力：

- 可读的班次与交接输入 connector baseline
- 能表达遗留事项与接收状态的 evidence / follow-up 对象
- 能表达交接超时与升级边界的 SLA 语义

在这三层补齐之前，`班次交接摘要与待办提取` 仍应被视为高价值、可快速扩面、但尚未进入正式实现闭环的高频协同 workflow。
