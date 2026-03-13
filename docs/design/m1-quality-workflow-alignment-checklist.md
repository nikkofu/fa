# M1 Quality Workflow Alignment Checklist

## 1. 文档目的

本文件把候选 workflow `质量偏差隔离与处置建议` 从 specification baseline 推进到实现对齐层，回答 5 个问题：

1. 当前平台已经有哪些能力可直接复用。
2. 哪些连接器必须先补，哪些可以延后。
3. 当前 `TaskEvidence / governance / audit` 能表达什么，不能表达什么。
4. 当前 API 是否足以支撑第一轮质量场景演示。
5. 后续应按什么顺序进入实现，而不是一口气把质量平台做全。

本文件仍然不是实现说明，更不是“质量 workflow 已经打通”的声明。它是后续 connector、evidence、governance 和 API 演进的对齐清单。

## 2. 对齐原则

质量场景进入实现前，继续坚持以下原则：

- 先做只读证据汇聚与建议，不做自动质量裁决。
- 先把责任边界和审批角色固定，再讨论写入 QMS / WMS / ERP。
- 先用 mock connector 验证任务、证据、审批、审计主链，再接真实系统。
- 先让 API 能完整回放“偏差 -> 证据 -> 建议 -> 审批 -> 跟踪”链路，再扩展质量专属端点。

## 3. 当前平台基线快照

| 对象 | 当前状态 | 对质量场景的意义 |
| --- | --- | --- |
| `TaskRequest` | 已支持通用任务字段、角色、风险、集成目标、期望结果 | 能发起质量任务，但还没有 `deviation_id / lot / batch / material` 等结构化字段 |
| `TrackedTaskState` | `tasks/intake` 会返回 `planned_task / context_reads / evidence / correlation_id` | 质量任务可以复用同一条任务主链 |
| `TaskEvidence` | 已支持 `connector / record_kind / source_ref / observed_at / summary / payload` | 可以先承接质量证据原始快照，但还不适合做 lot / batch 级结构化检索 |
| `WorkflowGovernance` | 已支持责任矩阵、审批策略和 fallback actions | 可复用治理框架，但当前审批角色仍偏设备 / 安全场景 |
| 审批校验 | 已支持 required role 强校验，角色不匹配时接口返回 `403` | 一旦引入 `Quality Manager` 策略，当前审批防越权机制可以直接复用 |
| connector registry | 默认只注册 mock `MES` / mock `CMMS` | 质量相关 target 虽已预埋，但还没有可读的 `QMS` 基线 |

## 4. Connector 对齐清单

### 4.1 最小必需连接器

| 系统 | 质量 workflow 最小必需记录 | 当前状态 | 建议 |
| --- | --- | --- | --- |
| `QMS` | 偏差编号、偏差状态、缺陷类别、严重度、处置状态、CAPA / NCR 关联 | 领域里已有 `IntegrationTarget::Quality`、`ConnectorKind::Quality`、`ConnectorRecordKind::QualityContext`，但默认未注册 connector，也没有读取计划 | Phase A 先补 mock `QMS` read-only connector，至少返回 1 组 deviation / inspection / disposition context |
| `MES` | 工单、工序、线体、设备、班次、批次 / lot 关联、在制品数量 | 已有 mock `MES` connector，但当前只返回 `TaskContext` 与设备温度 telemetry，偏向设备异常场景 | Phase A 同步扩展 MES mock 的质量上下文载荷，至少能表达工单与批次关联 |
| `ERP` | 订单影响、客户承诺、受影响成品 / 半成品、发运状态 | 已有 integration target，但没有默认 connector | 先不作为 Phase A 阻塞项；当影响范围涉及订单或客户交付时进入 mock baseline |
| `WMS` | lot 库存位置、隔离库存、待发货库存、仓位 | 已有 `Warehouse` target，但没有默认 connector | 若 workflow 只做到“建议隔离”，可晚于 QMS / MES；若要演示隔离对象范围，应在 Phase B 加 mock |
| `LIMS / SPC` | 检验结果、采样记录、控制限超规、趋势点 | 当前没有一等 `LIMS` / `SPC` integration target，可先通过 `Custom("lims") / Custom("spc")` 表达；默认也没有 connector | 只在企业确实有数据源时补；优先级低于 `QMS` 和 `MES` |

### 4.2 当前代码层面的关键差距

当前与质量连接器最相关的差距，不在“有没有 quality 这个词”，而在以下两点：

1. 默认 `ConnectorRegistry` 只注册了 mock `MES / CMMS`。
2. `requested_record_kinds()` 目前只会为 `MES` 和 `CMMS` 生成读取记录类型，`Quality / ERP / Warehouse / Custom` 目标当前不会拉回有效 records。

这意味着质量场景虽然在领域和枚举层面有预埋，但在运行主链上仍然是“可声明，不可读证据”。

### 4.3 Connector 实施顺序建议

1. `QMS` mock read-only baseline
2. `MES` 质量上下文补强
3. `WMS` mock inventory / quarantine context
4. `ERP` mock order impact context
5. `LIMS / SPC` 按数据可用性补入

## 5. Evidence 对齐清单

### 5.1 质量场景至少要表达的证据对象

| 证据对象 | 为什么必须有 | 当前是否能表达 | 差距 | 建议 |
| --- | --- | --- | --- | --- |
| `deviation_id / case_id` | 质量任务必须绑定正式偏差实体 | 只能放在 `title / description / payload` | 无一等字段，也无法直接按偏差号查询任务 | Phase A 先放入 payload；Phase B 再考虑质量元数据字段 |
| `lot / batch / serial / material` | 这是影响范围识别的核心 | 只能放在描述文本或 connector payload | 无结构化数组字段，难做 impact scope 展示 | Phase B 增加质量证据元数据或任务 intake 扩展字段 |
| `inspection_result / sample / spec breach` | 决定复检与升级建议 | 可以放进 `payload` 字符串 | 可存不可检索，`record_kind` 也过粗 | 先用 `QualityContext` 装载，后续再细化 record kind 或 typed evidence |
| `impact_scope` | 说明影响哪些批次、仓位、订单、客户 | 当前无一等对象 | 建议与实际执行范围无法清晰分开 | Phase C 形成结构化 impact scope 对象 |
| `containment_recommendation` | 隔离、复检、暂停放行建议的核心输出 | 目前只能放入自由文本说明 | 没有正式 recommendation 对象 | Phase C 增加 containment / disposition draft 结构 |
| `disposition taxonomy` | `hold / rework / scrap / use-as-is` 是质量语言基础 | 当前无模型 | 无法在治理、输出、审计里稳定复用 | 在 Phase C 定义只读 draft taxonomy，仍不自动执行真实动作 |
| 证据来源与时间戳 | 审计与可解释性需要 | 已有 `source_ref / observed_at / connector` | 基线足够 | 可直接复用 |

### 5.2 当前 `TaskEvidence` 能做什么

当前 `TaskEvidence` 的优点是：

- 足够通用，任何 connector 记录都能先进入任务 evidence
- 具备 `source_ref`、时间戳和 connector 来源
- 能作为 `tasks/intake` 与 `tasks/{task_id}/evidence` 的统一输出

当前 `TaskEvidence` 的局限是：

- `payload` 是字符串，不是质量域内可索引对象
- 缺少 lot / batch / material / inspection result 的结构化字段
- 无法区分“证据事实”和“待审批建议”
- 无法稳定表达“影响范围”和“建议处置”的结构化差异

### 5.3 Evidence 演进建议

建议把质量 evidence 演进拆成两层：

1. Phase A 继续复用当前 `TaskEvidence`，用 mock `QMS / MES` payload 证明 evidence 链条已通。
2. Phase B / C 再补质量元数据层，使 lot / batch / inspection / impact scope 成为正式对象，而不是长期塞在 JSON 字符串里。

## 6. Governance 对齐清单

### 6.1 角色与审批要求

| 治理项 | 当前状态 | 质量场景差距 | 建议 |
| --- | --- | --- | --- |
| accountable approver | 当前高风险任务默认落到 `Safety Officer`，极高风险升级到 `Plant Manager` | 质量场景正式审批人应是 `Quality Manager` 或等价质量责任人 | Phase B 新增 `Quality Manager` 审批策略或 workflow-specific policy resolver |
| consulted roles | 当前会按高风险或 `Quality` integration 自动加入 `quality_engineer`，设备场景可加入 `maintenance_engineer` | 质量场景还需要更清晰地区分 `Production Supervisor / Manufacturing Engineer / Warehouse Coordinator` 的 consulted / informed 边界 | 在质量 governance builder 中按 workflow 固化 RACI |
| decision scope | 当前 scope 文案偏“设备动作、维护执行、plant-level governance” | 质量场景需要覆盖 `batch hold / retest / shipment hold / CAPA escalation` | Phase B 增加质量专属 decision scope 模板 |
| escalation path | 当前只能表达 `plant_manager` 级升级 | 质量场景可能需要 `Plant Manager` 或 site-level quality head 的升级语义 | 先沿用 `plant_manager`，后续再决定是否引入质量组织层级 |
| forbidden actions | 当前没有真实写 connector，因此默认安全 | 质量场景仍需明确禁止自动放行、自动报废、自动库存冻结 | 在质量 workflow spec 和 connector policy 中继续保持 read-only baseline |
| role enforcement | 已支持 required role 强校验 | 只要 required role 还是 `safety_officer`，质量场景就会出现治理错配 | 在引入 `Quality Manager` 前，不应把质量场景宣称为正式审批闭环 |

### 6.2 与当前设备场景的关键差异

质量 workflow 与当前 pilot 最大的治理差异不是“也要审批”，而是：

- 审批责任主体不同
- 会签角色更偏质量、生产、仓储协同，而不是维护主导
- 决策对象是 `lot / batch / disposition`，不是设备维护动作
- 风险边界更接近质量放行、客户影响和库存影响

因此，不能简单复用 `Safety Officer` 语义，把它当作质量流程的长期审批替代。

## 7. API 对齐清单

### 7.1 当前已经可复用的接口

| 接口 | 当前是否可用于质量 workflow | 说明 |
| --- | --- | --- |
| `POST /api/v1/tasks/plan` | 可以 | 可先生成模式选择、计划步骤和治理草图 |
| `POST /api/v1/tasks/intake` | 可以 | 已返回 `planned_task / context_reads / evidence / correlation_id`，足以承接质量任务主链 |
| `GET /api/v1/tasks/{task_id}` | 可以 | 可查看任务状态、计划、审批记录与 evidence |
| `GET /api/v1/tasks/{task_id}/evidence` | 可以 | 适合 Phase A 查看原始质量证据快照 |
| `GET /api/v1/tasks/{task_id}/governance` | 可以 | 可验证质量场景是否进入正确角色与审批策略 |
| `GET /api/v1/tasks/{task_id}/audit-events` | 可以 | 可回放单任务质量事件轨迹 |
| `GET /api/v1/audit/events` | 可以 | 可按任务或链路主键过滤审计事件 |
| `POST /api/v1/tasks/{task_id}/approve` | 可以 | 一旦 required role 切到 `Quality Manager`，现有 `403` 防越权逻辑可继续复用 |
| `POST /api/v1/tasks/{task_id}/resubmit` | 可以 | 适合质量场景的补证据后重提 |
| `POST /api/v1/tasks/{task_id}/execute` / `complete` / `fail` | 可以 | 适合记录“隔离执行开始 / 完成 / 失败”，但当前仍是 generic stub |

### 7.2 当前 API 的主要差距

| 能力 | 当前状态 | 建议 |
| --- | --- | --- |
| 质量任务 intake 字段 | 通用字段足够发起任务，但没有 `deviation_id / lot_ids / material_ids / impacted_scope` | Phase B 再决定是否扩 request schema；Phase A 先用 payload 与描述字段承接 |
| 质量建议输出 | 当前没有 containment / disposition / CAPA draft 的专属对象 | Phase C 再补 structured draft output |
| 质量任务查询 | 当前只能按 task id / audit filters 查询 | 若要进入真实试运行，应支持按 `deviation_id / lot / batch` 检索 |
| evidence 过滤 | 当前是任务级全量 evidence 列表 | 后续可增加按 connector、record kind 或质量实体过滤 |

### 7.3 当前阶段不必新增的接口

在 Phase A 之前，不需要为了质量场景立刻新增专属 API。原因是当前接口已经足以验证：

- 质量任务能否进入生命周期主链
- 质量 connector 读取是否能进入 evidence
- 质量角色和审批策略是否能进入 governance
- 审计是否能完整回放

新的质量专属接口应在 draft output 和结构化质量元数据需要稳定暴露时再引入。

## 8. 分阶段实施建议

### Phase A. Mock QMS Read-only + Evidence Baseline

目标：

- 让质量任务首次拿到真实可见的质量证据，而不是只声明 `integrations: ["quality"]`

建议交付：

1. 注册 mock `QMS` / `Quality` connector 到默认 registry。
2. 让 `IntegrationTarget::Quality` 生成实际 `requested_records`。
3. 输出至少一组 `deviation / inspection / disposition` mock records。
4. 视需要扩展 mock `MES`，补批次 / 工单 / 在制品上下文。
5. 通过 `tasks/intake`、`tasks/{task_id}/evidence`、`audit-events` 完成 1 次只读质量任务演示。

退出标准：

- 质量任务的 evidence 不再为空。
- 审计里能看到 `QMS` 与 `MES` 的 connector read。
- 全流程仍然没有真实 QMS / WMS / ERP 写入。

### Phase B. Governance / Approval Strategy Mapping

目标：

- 让质量场景的审批责任从“通用高风险任务”升级到“质量责任人显式负责”

建议交付：

1. 引入 `Quality Manager` 或等价审批策略。
2. 为质量 workflow 生成专属 responsibility matrix。
3. 更新 decision scope，覆盖 `hold / retest / escalation` 等质量语义。
4. 保持现有 required-role enforcement，使错角色审批继续返回 `403`。

退出标准：

- `tasks/{task_id}/governance` 能返回质量专属审批角色和 RACI。
- 质量任务不再错误地落到 `Safety Officer` 作为默认审批人。

### Phase C. Draft Output for Containment / CAPA Proposal

目标：

- 让平台不只返回“读到了什么”，还能正式输出待审批质量建议

建议交付：

1. 定义 containment / disposition / CAPA proposal draft 结构。
2. 引入 impact scope 的结构化表达。
3. 让任务结果能区分“证据事实”、“建议动作”和“人工最终决定”。
4. 继续保持 draft-only，不自动执行真实质量放行、报废或库存冻结。

退出标准：

- 质量任务能从 intake 走到 draft recommendation。
- 用户能看清证据、建议、审批和执行跟踪之间的边界。

## 9. 当前最值得立即推进的实现项

如果只选 3 个最应该落地的动作，建议按以下顺序推进：

1. mock `QMS` read-only connector baseline
2. `Quality Manager` 审批策略与 governance 映射
3. 质量影响范围与 containment recommendation 的结构化表达

## 10. 结论

质量场景现在最缺的，不是新的任务接口，而是三件更基础的事：

- 真正能读到质量证据的 connector baseline
- 正确的质量责任人审批策略
- 能表达批次影响与处置建议的 evidence / output 结构

在这三件事补齐之前，`质量偏差隔离与处置建议` 仍应被视为“高价值、可继续推进、但尚未进入正式实现闭环”的候选 workflow。
