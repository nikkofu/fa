# M1 Alert Triage Workflow Alignment Checklist

## 1. 文档目的

本文件把 `产线告警聚合与异常分诊` 从 workflow specification baseline 推进到实现对齐层，重点回答 5 个问题：

1. 当前平台已经有哪些能力可以直接复用到告警分诊场景。
2. `SCADA / Andon / MES / incident log / CMMS` 分别缺什么 connector 基线。
3. 当前 `TaskEvidence`、governance 和 follow-up / SLA 模型能承接哪些告警对象，不能承接哪些。
4. 当前 API 是否足以支撑第一轮告警分诊演示。
5. 后续应按什么顺序进入实现，而不是把告警场景继续留在“事件驱动”口号层。

本文件不是实现说明，也不是“告警分诊已经打通”的声明。它是后续 connector、evidence、follow-up、governance、event-ingestion 和 query 演进的对齐清单。

## 2. 对齐原则

告警场景进入实现前，继续坚持以下原则：

- 先做只读告警读取、聚合和分诊草稿，不做自动消警、自动确认、自动停线。
- 先用 mock `SCADA / Andon / incident log` 验证主链，再讨论真实事件流接入。
- 先让 API 能回放 `告警时间窗 -> 多源读取 -> 聚合分诊 -> 升级候选` 链路，再扩展事件驱动入口。
- 普通告警分诊保持 confirmation / coordination 语义，高风险动作回到正式 governed workflow。
- Phase A 优先解决“有输入、有证据、有回放”，而不是一开始就追求实时流式架构。

## 3. 当前平台基线快照

| 对象 | 当前状态 | 对告警场景的意义 |
| --- | --- | --- |
| `TaskRequest` | 已支持标题、描述、优先级、风险、设备、集成目标和期望结果 | 能发起告警分诊任务，但没有 `alert_window / line_id / alert_source / cluster_key` 等结构化字段 |
| `TrackedTaskState` | `tasks/intake` 返回 `planned_task / context_reads / evidence / correlation_id` | 告警场景可以复用同一条任务主链 |
| `TaskEvidence` | 已支持 `connector / record_kind / source_ref / observed_at / summary / payload` | 能承接原始告警和现场上下文快照，但不适合稳定表达告警簇、时间窗和分诊状态 |
| `WorkflowGovernance` | 已支持责任矩阵、审批策略和 fallback actions | 适合表达升级边界，但普通告警分诊不是审批优先流程 |
| follow-up / SLA | 已有独立对齐清单，但尚无正式对象 | 告警分诊的 next step、owner 和时限仍无法结构化落地 |
| connector registry | `kind_for_target()` 能映射 `Scada`，但默认只注册 mock `MES / CMMS` | 告警任务即使声明 `scada`，当前默认运行也读不到真实 records |
| 事件入口 | 当前只有 `plan / intake` 请求式入口 | 还没有事件驱动 ingestion、聚类窗口和增量聚合视图 |

## 4. Connector 对齐清单

### 4.1 最小必需连接器

| 系统 | 告警 workflow 最小必需记录 | 当前状态 | 建议 |
| --- | --- | --- | --- |
| `SCADA` | 原始告警事件、设备/点位、严重度、触发/恢复时间、告警码 | `IntegrationTarget::Scada` 已存在，但默认 registry 没有 mock `Scada` connector | Phase A 补 read-only mock `Scada` baseline |
| `Andon` | 人工拉灯、工位状态色、班组求助、人工备注 | 当前没有一等 `IntegrationTarget::Andon`，可先用 `Custom("andon")` 表达 | Phase A 先补 mock `andon` custom connector |
| `MES` | 线体状态、工单、产量影响、设备上下文、停机上下文 | 已有 mock `MES`，但 payload 偏设备异常场景，只返回 `TaskContext / EquipmentTelemetry` | Phase A 扩展 line-level status、order impact、exception count |
| `Incident Log` | 现场登记、已知临时处置、未关闭异常、值班备注 | 当前没有一等 target，可先用 `Custom("incident_log")` 表达 | Phase A 与 `Andon` 一起补 mock baseline |
| `CMMS` | 历史维护异常、未关闭工单、已知故障模式 | 已有 mock `CMMS`，但返回内容偏设备异常上下文 | Phase A 可直接复用，再补未关闭事项语义 |

### 4.2 当前代码层面的关键差距

当前与告警连接器最相关的差距主要有 5 点：

1. 默认 `ConnectorRegistry` 只注册了 mock `MES / CMMS`。
2. `IntegrationTarget` 有 `Scada`，但没有 `Andon` 和 `incident_log` 这类一等目标。
3. `requested_record_kinds()` 当前只为 `MES` 和 `CMMS` 生成读取记录，其他 target 会得到空的 `requested_records`。
4. `ConnectorRecordKind` 当前没有 `AlertEvent / AlertCluster / AlertTimeline / TriageDecision` 这类正式对象，只能先借助 `Custom(_)`。
5. `primary_subject()` 当前优先取第一条 `equipment_id`，并不能表达“线体 + 时间窗 + 多源告警簇”这类更适合告警场景的 subject。

这意味着告警场景虽然已经有 spec，但在运行主链上仍然缺少真正可读的多源告警输入。

### 4.3 Connector 实施顺序建议

1. mock `Scada` read-only baseline
2. mock `andon` read-only baseline
3. mock `incident log` read-only baseline
4. 扩展 mock `MES` 的 line / order impact payload
5. `CMMS` 未关闭工单与相似故障语义补强

## 5. Evidence 与分诊输出对齐清单

### 5.1 告警场景至少要表达的对象

| 对象 | 为什么必须有 | 当前是否能表达 | 差距 | 建议 |
| --- | --- | --- | --- | --- |
| `alert_cluster_id / grouping_key` | 告警分诊的核心不是单条告警，而是同一异常簇 | 只能塞进 `payload` 文本 | 没有稳定 id，难以检索和回放 | Phase A 先放入 JSON payload；Phase B 再考虑正式读模型 |
| `source_alert_refs` | 必须知道一个簇由哪些原始告警组成 | 单条 `TaskEvidence` 只有一个 `source_ref` | 多源引用无法结构化展开 | Phase A 先放入 payload 数组 |
| `alert_window / timeline` | 告警分诊依赖时间窗、先后顺序和持续时间 | 只有单条 `observed_at` 时间点 | 无法表达窗口、恢复时间和时间线 | Phase A 先用 payload 时间线；Phase B 再补 timeline-style evidence |
| `severity / triage_label` | 需要区分提醒、关注、升级候选等分诊层级 | 只能写在摘要文本里 | 没有枚举或可查询状态 | Phase B 引入正式 triage output 字段 |
| `escalation_candidate` | 需要明确哪些簇必须进入正式异常任务 | 只能放在自然语言说明里 | 无法稳定区分建议与已升级 | 依赖 governance 和 follow-up 模型共同补齐 |
| `recommended_owner / next_step` | 告警分诊不是总结结束，而是要进入后续动作 | 只能写在自由文本中 | 无法与后续 follow-up 绑定 | 依赖 follow-up / SLA 模型 |

### 5.2 当前 `TaskEvidence` 能做什么

当前 `TaskEvidence` 的优点是：

- 能快速承接 `SCADA / Andon / MES / incident log / CMMS` 的读取结果
- 具备 connector、record kind、来源引用和时间戳
- 可以通过 `tasks/intake`、`tasks/{task_id}`、`tasks/{task_id}/evidence` 一致输出
- 能先用 `payload` 承接 mock alert group、timeline 和 triage draft JSON

### 5.3 当前 `TaskEvidence` 的局限

当前 `TaskEvidence` 的局限是：

- 更像离散 snapshot，而不是告警时间线或告警簇对象
- `payload` 是字符串，适合展示，不适合稳定查询和聚合
- 一条 evidence 只有一个 `source_ref`，不适合多源簇引用
- 不能表达 triage label、escalation state、owner assignment 的正式状态变化

### 5.4 与 follow-up / SLA 模型的关系

告警分诊不是在摘要层结束。它至少会继续产生：

- 一个或多个 next step
- recommended owner
- due window 或响应时限
- escalation candidate

因此告警场景虽然以事件为起点，但后半段仍然会落到 follow-up / SLA 模型上。如果没有这层对象，告警分诊就只能停留在“建议怎么做”，无法进入后续跟踪。

## 6. Governance 对齐清单

### 6.1 角色与边界

| 治理项 | 当前状态 | 告警场景差距 | 建议 |
| --- | --- | --- | --- |
| responsibility matrix | 当前 governance 已能输出 `Responsible / Accountable / Consulted / Informed` | 可以复用，但需把 `Production Supervisor / Shift Lead / Maintenance / Quality / Safety` 的 triage 边界收紧 | 在 alert triage governance builder 中固化默认参与角色 |
| triage confirmation | 当前平台更擅长审批门控 | 普通告警分诊更偏确认与接收，不是正式审批 | 先保持 confirmation / coordination 语义 |
| formal approval | 当前 approval strategy 适合高风险任务 | 告警分诊本身不该默认进入审批 | 只在安全、质量、停线升级后进入 governed workflow |
| escalation boundary | 当前 fallback actions 可表达升级说明 | 还不能稳定表达“哪些 triage 结果必须转正式任务” | 依赖 triage label + follow-up / SLA 补齐 |
| forbidden actions | 当前无真实写 connector，因此默认安全 | 仍需明确禁止自动停线、自动消警、自动安全裁决 | 保持 read-only / draft baseline |

### 6.2 为什么当前 generic governance 只部分适用

当前 generic governance 的优势是：

- 能表达责任矩阵
- 能表达高风险审批角色
- 能表达 fallback actions

但它对普通告警分诊只部分适用，因为：

- triage 结果首先是“建议”和“确认”，不是“批准”
- 告警场景最重要的是升级边界，而不是审批流程本身
- 安全、质量、停线风险应从 triage 任务跳转到正式 governed workflow，而不是把 triage 本身包装成审批流

## 7. API 对齐清单

### 7.1 当前已经可复用的接口

| 接口 | 当前是否可用于告警 workflow | 说明 |
| --- | --- | --- |
| `POST /api/v1/tasks/plan` | 可以 | 可先生成模式选择、治理草图和步骤建议 |
| `POST /api/v1/tasks/intake` | 可以 | 已能返回 `planned_task / context_reads / evidence / correlation_id` |
| `GET /api/v1/tasks/{task_id}` | 可以 | 可查看任务状态、计划和 evidence |
| `GET /api/v1/tasks/{task_id}/evidence` | 可以 | 适合查看告警原始线索和聚合草稿 |
| `GET /api/v1/tasks/{task_id}/governance` | 可以 | 可验证责任矩阵和升级边界 |
| `GET /api/v1/tasks/{task_id}/audit-events` | 可以 | 可回放告警分诊轨迹 |

### 7.2 当前 API 的主要差距

| 能力 | 当前状态 | 建议 |
| --- | --- | --- |
| event-driven ingestion | 当前只有请求式 intake，没有受控事件入口 | Phase C 再收敛事件适配器或 ingestion endpoint |
| alert intake 字段 | 通用字段够发起任务，但没有 `alert_window / cluster_key / line_id / source_systems` | Phase A 先放入 payload；Phase B 再决定是否扩 request schema |
| alert cluster read model | 当前只能读 task 级 evidence | Phase B 补 `alert cluster / triage draft` 读模型 |
| cross-task cluster query | 当前审计只支持 `task_id / correlation_id / kind / approval_id` 过滤 | Phase C 再补按 cluster、设备、时间窗查询 |
| follow-up / owner / SLA | 当前无法结构化返回 triage 后续动作 | 依赖 follow-up / SLA 模型进入 API |

### 7.3 当前阶段不必立即新增的接口

在 connector、evidence payload 和 triage output 语义未冻结前，不需要立刻新增一批告警专属 endpoint。

更合理的顺序是：

1. 先让告警任务通过现有任务主链拿到真实多源 evidence。
2. 再让 alert cluster / triage draft 进入任务读模型。
3. 最后再决定是否需要独立的事件 ingestion 与聚合查询接口。

## 8. 分阶段实施建议

### Phase A. Mock SCADA / Andon / Incident Log + Evidence Baseline

目标：

- 让告警任务第一次拿到真实可见的多源告警证据，而不只是人工描述

建议交付：

1. 注册 mock `Scada` read-only connector。
2. 注册 mock `Custom("andon")` / `Custom("incident_log")` connector。
3. 扩展 `requested_record_kinds()`，让这些 target 不再返回空读取集合。
4. 扩展 mock `MES`，补 line status、order impact、exception count 上下文。
5. 让告警任务通过 `tasks/intake` 和 `tasks/{task_id}/evidence` 展示多源原始 evidence。

退出标准：

- 告警任务的 evidence 不再只来自人工描述。
- 审计中能看到 `SCADA / Andon / incident log / MES` 的 connector reads。

### Phase B. Alert Cluster and Triage Draft Baseline

目标：

- 让告警输出从“若干条原始证据”推进到“可理解的告警簇与分诊草稿”

建议交付：

1. 冻结最小 `alert cluster` payload schema。
2. 输出 triage label、severity、recommended owner 和 escalation candidate。
3. 让一个告警任务能包含多个 cluster draft，而不是单条摘要。
4. 把 triage next step 与 follow-up / SLA read model 对接。

退出标准：

- 告警任务能稳定说明哪些告警被归并、为什么被提级、由谁接手。

### Phase C. Event Ingestion and Aggregation / Query Direction

目标：

- 让告警场景真正具备“事件到任务”的受控转换能力

建议交付：

1. 定义受控事件 ingestion 入口或适配器边界。
2. 定义去重窗口、聚类 key 和重放边界。
3. 补按 cluster、设备、时间窗查看 triage history 的聚合视图。
4. 补高时效告警的 follow-up / SLA 升级联动。

退出标准：

- 用户能从单次任务回放走到跨告警簇、跨时间窗的聚合查询。

## 9. 当前最值得立即推进的实现项

如果只选 4 个最该落地的动作，建议顺序如下：

1. mock `Scada / andon / incident_log` read-only baseline
2. 扩展 `requested_record_kinds()` 和 `ConnectorRecordKind::Custom(_)` 的告警语义
3. 冻结 `alert cluster / triage draft` payload schema
4. 开始收敛 follow-up / SLA read model 与 query 方向

## 10. 结论

告警场景当前最缺的，不是新的总结提示词，而是三层更基础的能力：

- 可读的多源告警输入 connector baseline
- 能表达告警簇、时间线和升级候选的 evidence / triage 对象
- 能把分诊结果继续接到 follow-up / SLA 的后续协同层

在这三层补齐之前，`产线告警聚合与异常分诊` 仍应被视为高价值、强时效、适合事件驱动扩展，但尚未进入正式实现闭环的高频协同 workflow。
