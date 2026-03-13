# M1 Alert Triage Alert Cluster and Event Ingestion Direction

## 1. 文档目的

本文件把 `产线告警聚合与异常分诊` 从 alignment checklist 继续推进到 `alert cluster / event-ingestion` 专属方向。

它重点回答 5 个问题：

1. 为什么 `raw alert`、`alert cluster`、`triage draft` 和 `follow-up item` 不能混成一个对象。
2. `alert cluster` 最适合先如何进入 task detail，再如何进入跨任务聚合查询。
3. 事件应在什么边界上进入平台，而不是一条原始告警就直接变成任务。
4. 当前任务主链、evidence、audit 和 API 基线能先承接什么，不能承接什么。
5. 后续实现应按什么顺序推进，而不是把 event-driven 继续停留在抽象口号层。

本文件不是代码实现说明，也不是“告警事件入口已经存在”的声明。它是后续告警专属 read model、受控 ingestion 边界和聚合查询方向的说明。

## 2. 方向原则

告警场景进入 `alert cluster / event-ingestion` 设计时，继续坚持以下原则：

- 原始告警事件不直接等于 triage task。
- `alert cluster` 不直接等于 `follow-up item`。
- 先把 cluster draft 进入 task detail，再做跨时间窗 cluster query。
- 先做受控 ingestion 边界，不做开放式“所有告警都自动进系统”的入口。
- event ingestion 只负责接收、规范化、去重和归并候选，不负责自动停线、自动消警或自动裁决。
- 高风险动作仍回到 governed workflow，event-driven 不是越过治理的理由。

## 3. 为什么这层必须单独拆出来

告警场景里至少有 4 层不同对象：

1. `raw alert event`
2. `alert cluster`
3. `triage draft`
4. `follow-up item`

它们分别回答不同问题：

- `raw alert event`：到底发生了什么信号
- `alert cluster`：哪些信号属于同一异常簇
- `triage draft`：这个簇应该怎样分级、路由和升级
- `follow-up item`：这个分诊结果后续到底谁来跟、何时完成

如果把四层对象混在一起，后续实现就会出现 3 个问题：

1. 一条原始告警就可能错误触发一条完整任务。
2. 聚类和分诊理由会重新被塞回自由文本。
3. follow-up 查询会混入大量还未成形的原始信号噪音。

## 4. 当前平台基线快照

| 对象 | 当前状态 | 对 alert cluster / ingestion 的意义 |
| --- | --- | --- |
| `TaskRequest` | 只有通用任务字段，没有 `alert_window / line_id / source_system / cluster_key` | 当前只适合承接人工触发或受控生成的 triage task |
| `TrackedTaskState` | `tasks/{task_id}` 返回 `planned_task / context_reads / evidence / correlation_id` | task detail 已是 cluster draft 最自然的第一层挂载点 |
| `TaskEvidence` | 已支持来源、时间点、摘要和字符串 payload | 适合先承接 cluster draft JSON，但不适合长期承担 cluster 查询 |
| `ConnectorRegistry` | 默认只注册 mock `MES / CMMS` | event-driven 输入目前还没有默认 `Scada / Andon` 读取基线 |
| `requested_record_kinds()` | 只为 `MES / CMMS` 生成读取请求 | event source target 目前没有正式读取语义 |
| API 主链 | 当前只有 `tasks/plan`、`tasks/intake`、`tasks/{task_id}`、`evidence`、`governance`、`audit-events` | 当前没有独立 ingestion 或 cluster query 资源 |
| audit 主链 | 仅支持 `task_id / approval_id / correlation_id / kind` 过滤 | 适合回放 triage 过程，不适合直接做 cluster list 查询 |

## 5. 推荐的对象拆分

### 5.1 Raw Alert Event

`raw alert event` 先表达最基础的外部信号。

建议最小字段：

- `source_system`
- `source_event_id`
- `equipment_id` 或 `line_id`
- `alert_code`
- `severity`
- `state`
- `observed_at`
- `cleared_at`
- `payload`

这层目标是“忠实接收”，不是“理解业务”。

### 5.2 Alert Cluster

`alert cluster` 解决的是归并问题。

建议最小字段：

- `cluster_id`
- `cluster_key`
- `source_systems`
- `equipment_id` / `line_id`
- `window_start`
- `window_end`
- `open_event_count`
- `repeated_event_count`
- `dominant_alert_code`
- `severity_band`
- `source_event_refs`
- `cluster_status`

这层回答的是：

- 哪些原始告警属于同一异常簇
- 这个簇当前是否还在持续
- 这个簇的严重度和重复度大致如何

### 5.3 Triage Draft

`triage draft` 解决的是判断问题。

建议最小字段：

- `cluster_id`
- `triage_label`
- `business_impact`
- `recommended_owner_role`
- `recommended_next_step`
- `escalation_candidate`
- `requires_quality_review`
- `requires_safety_review`
- `rationale`
- `drafted_at`

这层回答的是：

- 这个簇现在该怎样看
- 应该先找谁
- 是否需要升级

### 5.4 Follow-up Item

`follow-up item` 解决的是后续执行问题。

它不应与 cluster 或 triage draft 合并。

更合理的关系是：

- `follow_up_item.source_kind = alert_cluster`
- `follow_up_item.source_refs` 指向 `cluster_id` 或 triage draft
- 一个 cluster 可以生成多个 follow-up items

## 6. Task detail 与 query 方向

### 6.1 Task detail 方向

Phase A 最适合先在 `GET /api/v1/tasks/{task_id}` 中增加：

- `alert_cluster_drafts`
- `alert_triage_summary`

其中 `alert_cluster_drafts` 建议至少包含：

- `cluster_id`
- `cluster_status`
- `severity_band`
- `source_event_refs`
- `window_start`
- `window_end`
- `triage_label`
- `recommended_owner_role`
- `escalation_candidate`

`alert_triage_summary` 则至少应包含：

- `total_clusters`
- `open_clusters`
- `high_priority_clusters`
- `escalation_candidate_count`
- `last_clustered_at`

### 6.2 为什么先放任务详情

原因很直接：

1. 当前任务详情主链已经稳定。
2. `TrackedTaskState` 在 file / SQLite 模式下都直接全量持久化。
3. cluster draft 先进入 task detail，能最快和现有 evidence、governance、audit 打通。
4. 这一步不要求马上引入真实 ingestion 或聚合查询资源。

### 6.3 跨任务 query 方向

当 cluster draft 进入 Phase B 后，才值得考虑聚合查询资源，例如：

- `GET /api/v1/alert-clusters`

第一版最值得支持的过滤维度包括：

- `cluster_status`
- `source_system`
- `equipment_id`
- `line_id`
- `severity_band`
- `triage_label`
- `escalation_candidate`
- `window_from`
- `window_to`
- `open_only`

这层主要回答：

- 现在线上有哪些正在持续的异常簇
- 哪些簇需要优先看
- 哪些簇已经多次重复但还没升级

## 7. Event ingestion 边界方向

### 7.1 不推荐的错误做法

最不该走的路有两条：

1. 每来一条原始告警就立即创建一条 triage task。
2. 把原始告警直接塞进 `tasks/intake`，让 task schema 承担 event envelope 职责。

这样会导致任务泛滥、噪音过高、聚类窗口消失，也很难保持治理边界。

### 7.2 推荐的受控链路

更合理的受控链路是：

1. 接收 `raw alert event`
2. 先做 normalization
3. 做去重 / 抑制 / 短窗口归并
4. 生成或更新 `alert cluster candidate`
5. 满足阈值后才生成或刷新 `triage task`
6. 再从 triage draft 生成 follow-up 和升级候选

### 7.3 第一版 ingestion 形态建议

第一版不必急着做完全开放的实时入口。

更合理的选择顺序是：

1. 先用 mock `Scada / Andon / incident_log` connector 通过受控读取生成 cluster draft。
2. 再设计内部 ingestion adapter 或受控入口。
3. 最后再决定是否开放独立 ingestion API。

### 7.4 如果后续增加入口，最适合的资源方向

当对象和边界稳定后，后续最可能的资源方向是：

- `POST /api/v1/alert-events/ingest`

但它应明确是：

- 受控 ingestion 入口
- 输入原始事件 envelope
- 返回 ingestion decision 或 cluster association

而不是直接返回“任务已创建并已分诊完成”。

## 8. Governance 与审计方向

### 8.1 Governance 边界

alert cluster / event-ingestion 最重要的治理边界是：

- event ingestion 只负责接收和归并，不负责业务裁决
- triage draft 是建议，不是批准
- `Production Supervisor` 负责确认现场影响
- `Safety Officer / Quality Engineer` 只在需要时进入正式升级路径
- 自动停线、自动消警、自动安全裁决仍然禁止

### 8.2 后续最值得补的审计事件

告警 cluster 进入实现后，至少应支持：

- `alert_event_ingested`
- `alert_event_suppressed`
- `alert_cluster_created`
- `alert_cluster_updated`
- `alert_cluster_escalation_flagged`
- `alert_triage_drafted`
- `alert_triage_confirmed`

这些事件更适合支撑：

- cluster 回放
- triage 解释
- projection rebuild

而不是直接替代 cluster query 本身。

## 9. 分阶段实施建议

### Phase A. Task-scoped Cluster Draft Baseline

目标：

- 让告警任务第一次拥有正式的 cluster draft 读模型

建议交付：

1. 冻结 `alert_cluster_draft` 最小字段集。
2. 让 `TaskEvidence.payload` 能稳定承接 cluster draft JSON。
3. 在 `tasks/{task_id}` 中增加 `alert_cluster_drafts / alert_triage_summary`。
4. 保持所有对象为 read-only / draft 输出。

退出标准：

- 用户能在任务详情中看见“哪些告警被归并为哪些簇”。

### Phase B. Controlled Event Ingestion and Cluster Projection

目标：

- 让平台具备受控的事件到 cluster 的进入能力

建议交付：

1. 定义 ingestion envelope 和 normalization 边界。
2. 定义 cluster key、去重窗口和 suppress 规则。
3. 定义何时新建 triage task，何时更新现有 cluster。
4. 输出 cluster 级 projection，而不是只依赖 task detail。

退出标准：

- 用户能解释一条原始告警为什么进入了某个 cluster，或者为什么被抑制。

### Phase C. Cross-window Query and Scenario Linkage

目标：

- 让 alert cluster 进入真正的平台级事件协同视图

建议交付：

1. 输出 `alert-clusters` 聚合查询。
2. 支持 ongoing cluster、repeated cluster 和 escalation candidate 查询。
3. 让 cluster 与 follow-up / SLA query 联动。
4. 让 event-ingestion、cluster、triage、follow-up 构成一条可回放链路。

退出标准：

- 用户能从单次 triage task 走到跨时间窗 cluster backlog 视图。

## 10. 当前最值得立即推进的动作

如果只选 4 个最该落地的动作，建议顺序如下：

1. 冻结 `alert_cluster_draft` 最小字段和状态语义
2. 明确 `raw alert`、`cluster`、`triage draft`、`follow_up_item` 的边界
3. 明确第一版受控 ingestion 的 normalization 和 suppress 规则边界
4. 定义 task detail 中 `alert_cluster_drafts / alert_triage_summary` 的字段集

## 11. 结论

告警场景真正需要的，不只是更多 evidence，而是一层独立的 `alert cluster / event-ingestion` 对象来表达：

- 原始告警如何进入平台
- 哪些信号被归并为同一异常簇
- 哪些簇已经形成正式 triage draft
- 哪些簇进一步生成了 follow-up 和升级候选

只有把这层对象单独做出来，`产线告警聚合与异常分诊` 才会从“有证据、有建议”进一步走向“有事件入口、有聚合对象”的高频事件协同能力。
