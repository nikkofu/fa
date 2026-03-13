# M1 Quality QMS Mock Connector Baseline

## 1. 文档目的

本文件把 `质量偏差隔离与处置建议` 从 workflow alignment checklist 继续推进到 `mock QMS` connector baseline 方向。

它重点回答 5 个问题：

1. 当前代码里 `Quality / QMS` 已经预埋到了哪一层，哪一层还没有真正接通。
2. 第一版 mock `QMS` connector 最少应该返回哪些记录，而不是一开始就假装做完整质量平台。
3. 第一版 payload 应如何与当前 `ConnectorRecordKind::QualityContext`、`TaskEvidence`、审计和任务主链对齐。
4. mock `QMS` baseline 应如何和现有 mock `MES` 形成最小联动，而不是孤立输出质量文本。
5. 后续实现应按什么顺序推进，而不是直接跳到真实写入或复杂处置 taxonomy。

本文件不是代码实现说明，也不是“QMS connector 已经存在”的声明。它是后续 `mock QMS` read-only connector、evidence baseline 和任务演示路径的方向说明。

## 2. 方向原则

`QMS` mock baseline 继续坚持以下原则：

- 先做 read-only evidence baseline，不做真实质量写入。
- 先让 `tasks/intake -> evidence -> governance -> audit` 主链拿到非空质量证据，再讨论质量专属端点。
- 先复用现有 `ConnectorRecordKind::QualityContext`，不在第一版急着扩很多 record kinds。
- payload 要能表达质量语义，但不要求第一版就可结构化检索所有质量实体。
- `QMS` baseline 要和 `MES` 质量上下文形成最小联动，避免只返回孤立偏差文本。
- 审批责任仍单独留给后续 `Quality Manager` governance 映射，不在 connector note 里混写成已解决问题。

## 3. 当前代码基线快照

| 对象 | 当前状态 | 对 `QMS` baseline 的意义 |
| --- | --- | --- |
| `IntegrationTarget::Quality` | 已存在 | 任务请求已经能声明质量集成目标 |
| `ConnectorKind::Quality` | 已存在 | connector 类型已经预埋，说明领域词汇层没有缺口 |
| `ConnectorRecordKind::QualityContext` | 已存在 | 第一版最适合先复用这个 record kind 承接 `QMS` records |
| `ConnectorRegistry::with_m1_defaults()` | 当前只注册 mock `MES / CMMS` | 即使任务声明 `quality`，默认运行链路里也读不到 `QMS` 记录 |
| `requested_record_kinds()` | 当前只为 `MES / CMMS` 返回 record kinds | `IntegrationTarget::Quality` 现在会得到空读取列表 |
| `TaskEvidence` | 已支持 connector、record kind、source ref、payload、observed_at | 第一版 `QMS` 记录可以无缝进入 evidence snapshot |
| governance builder | 当任务声明 `Quality` integration 时，会自动加入 `quality_engineer` consulted 角色 | 说明 connector baseline 接通后，质量任务至少能在责任矩阵里看到质量角色参与 |
| approval policy | 仍只有 `Auto / OperationsSupervisor / SafetyOfficer / PlantManager` | `Quality Manager` 审批策略还没有解决，不应在本阶段混淆为已打通 |

## 4. 为什么现在先做 `mock QMS`

当前质量场景最大的现实问题，不是没有 `Quality` 这个词，而是：

- 任务可以声明 `integrations: ["quality"]`
- 但运行时默认读不到任何 `QMS` 记录
- evidence 无法展示正式质量证据来源
- 审计里也看不到 `QMS` connector read

这会导致质量场景虽然在文档层已对齐，但运行链路仍然是“可声明，不可读证据”。

因此，最合理的下一步不是先扩新的质量接口，而是先让默认运行主链第一次读到 `QMS` baseline evidence。

## 5. 推荐的最小 `QMS` 记录集

### 5.1 第一版必须有的记录

第一版 mock `QMS` connector 建议最少返回 3 类记录，全部先复用 `ConnectorRecordKind::QualityContext`：

| `record_type` | 作用 | 为什么第一版就要有 |
| --- | --- | --- |
| `deviation_case` | 表达偏差编号、严重度、状态、缺陷类别 | 这是质量任务的主实体，没有它就不像正式质量场景 |
| `inspection_summary` | 表达检验结果、超规项目、采样或复检状态 | 没有检测上下文就很难支撑“为什么建议隔离 / 复检” |
| `disposition_context` | 表达当前处置状态、是否已 hold、是否已有 CAPA / NCR 关联 | 没有处置上下文就很难区分“仅发现问题”和“已经进入处置流程” |

### 5.2 第一版可以延后的记录

以下记录不建议作为 Phase A 阻塞项：

- `customer_impact`
- `shipment_hold`
- `wms_quarantine_location`
- `spc_trend_point`
- `lims_sample_detail`

这些信息有价值，但它们更适合在 `MES / ERP / WMS / LIMS` 联动阶段再补，不应阻塞第一版 `QMS` baseline。

## 6. 推荐的 payload 基线

### 6.1 为什么先复用 `QualityContext`

当前代码只有一个质量相关 record kind：`ConnectorRecordKind::QualityContext`。

因此 Phase A 最合理的做法不是立刻扩 enum，而是：

- 继续返回 `QualityContext`
- 在 payload 内增加 `record_type`
- 用 `source_ref` 明确 record 来源

这样可以最快接通默认主链，也不会过早冻结一堆质量 record kinds。

### 6.2 建议的 payload 示例

`deviation_case` 示例：

```json
{
  "record_type": "deviation_case",
  "deviation_id": "DEV-20260312-001",
  "severity": "major",
  "status": "open",
  "defect_category": "dimension_out_of_spec",
  "lot_id": "LOT-20260312-A1",
  "work_order": "MO-20260312-08"
}
```

`inspection_summary` 示例：

```json
{
  "record_type": "inspection_summary",
  "inspection_id": "INSP-20260312-015",
  "result": "fail",
  "spec_breach": "diameter_gt_upper_limit",
  "sample_size": 5,
  "failed_count": 2,
  "retest_required": true
}
```

`disposition_context` 示例：

```json
{
  "record_type": "disposition_context",
  "disposition_status": "containment_pending",
  "hold_recommended": true,
  "capa_id": null,
  "ncr_id": "NCR-20260312-003",
  "owner_role": "quality_engineer"
}
```

### 6.3 `source_ref` 建议

建议第一版使用清晰的 `source_ref` 前缀，例如：

- `qms://deviations/DEV-20260312-001`
- `qms://inspections/INSP-20260312-015`
- `qms://dispositions/NCR-20260312-003`

这样当前 `TaskEvidence` 和审计回放已经能清楚展示质量证据来源。

## 7. 与现有主链的对齐方式

### 7.1 Registry 对齐

第一版最直接的落点是：

1. 在 `ConnectorRegistry::with_m1_defaults()` 注册 `MockQualityConnector`
2. 让 `IntegrationTarget::Quality` 在默认运行模式下不再是“空 target”

### 7.2 Read plan 对齐

当前 `requested_record_kinds()` 对 `IntegrationTarget::Quality` 返回空列表。

Phase A 最小变更应是：

- `IntegrationTarget::Quality => vec![ConnectorRecordKind::QualityContext]`

这足以让 read request 成为真正有效的质量读取，而不必先引入新的 quality-specific record kinds。

### 7.3 Evidence 对齐

当前 `TaskEvidence` 已经足够承接第一版质量证据：

- `connector = Quality`
- `record_kind = QualityContext`
- `source_ref = qms://...`
- `payload = JSON string`

现有 `evidence_from_context_reads()` 也已经为 `QualityContext` 提供了基础 summary 输出，因此 Phase A 不需要先改 evidence pipeline 才能看见质量证据。

### 7.4 Audit 对齐

当前 orchestrator 在每次 connector read 后都会写入 `ConnectorRead` 事件。

一旦 `mock QMS` baseline 接通，现有审计主链就能自然开始记录：

- `Read N context records from Quality`

这正是第一版最重要的可验证信号之一。

## 8. 与 mock `MES` 的最小联动

单独的 `QMS` baseline 仍然不够，因为质量场景至少需要最小生产上下文来解释：

- 这个偏差关联哪个工单
- 关联哪个 lot / batch
- 当前在制品或工单状态如何

因此更合理的 Phase A 组合是：

- `QMS` 提供 `deviation / inspection / disposition`
- `MES` 提供 `work_order / line / batch / production_status`

第一版不必把 mock `MES` 扩到完整质量模型，但至少应从“设备温度 telemetry”补到“工单 + 批次关联”。

## 9. 当前阶段故意不解决的问题

本阶段不应假装已经解决以下问题：

- `Quality Manager` 审批策略
- `hold / rework / scrap / use-as-is` taxonomy 的正式对象
- `ERP / WMS` 订单与库存影响
- `LIMS / SPC` 检验趋势
- lot / batch 级结构化 query
- 真实 `QMS` 写入或 CAPA / NCR 创建

这些都重要，但不应阻塞第一版 `QMS` read-only baseline。

## 10. 分阶段实施建议

### Phase A. Mock QMS Read-only Baseline

目标：

- 让质量任务第一次拿到真实可见的 `QMS` evidence

建议交付：

1. 注册 `MockQualityConnector`
2. 让 `IntegrationTarget::Quality` 返回 `QualityContext` 读取计划
3. 返回 `deviation_case / inspection_summary / disposition_context` 三类 payload
4. 通过 `tasks/intake`、`tasks/{task_id}/evidence`、`audit-events` 完成 1 次质量任务演示

退出标准：

- `integrations: ["quality"]` 的任务 evidence 不再为空
- 审计中能看到 `Quality` connector read

### Phase B. QMS + MES Quality Evidence Pair

目标：

- 让质量任务第一次同时看见质量和生产上下文

建议交付：

1. 扩展 mock `MES` 的质量相关 payload
2. 让 quality task evidence 同时出现 `QMS` 与 `MES`
3. 让 evidence 能解释“偏差是什么”和“影响哪个工单 / 批次”

退出标准：

- 用户能从任务 evidence 同时看见 `deviation` 和 `work_order / lot` 关联

### Phase C. Governance and Draft Output Linkage

目标：

- 让质量证据开始进入治理和建议输出链条

建议交付：

1. 把 `Quality Manager` 审批策略映射接上
2. 开始定义 containment / disposition draft 结构
3. 区分证据事实、建议动作和人工最终决定

退出标准：

- 质量任务不再只是“读到了质量记录”，而能走到正式 draft recommendation

## 11. 当前最值得立即推进的动作

如果只选 4 个最该落地的动作，建议顺序如下：

1. 注册 `MockQualityConnector` 到默认 registry
2. 让 `IntegrationTarget::Quality` 返回 `QualityContext` 读取计划
3. 冻结 `deviation_case / inspection_summary / disposition_context` payload baseline
4. 扩展 mock `MES` 的最小质量上下文，补工单 / lot 关联

## 12. 结论

质量场景现在最缺的，不是再写一版 workflow 描述，而是让默认运行主链第一次真正读到 `QMS` 证据。

第一版最合理的做法，不是追求完整质量平台，而是：

- 用 `mock QMS` 接通 `Quality` target
- 先复用 `QualityContext`
- 让 evidence 和 audit 里出现正式质量证据来源
- 再逐步把 `MES`、governance 和 draft output 接上

只有这样，`质量偏差隔离与处置建议` 才会从“已对齐的候选场景”继续推进到“可演示的证据链路”。
