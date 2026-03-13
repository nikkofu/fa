# M1 Alert Triage Task Read Model Implementation Cut

## 1. 文档目的

本文件把 `alert cluster / triage draft` 从方向说明继续推进到“最小可实施代码切口”。

它重点回答 5 个问题：

1. 最小 `alert_cluster_drafts` 应先怎样进入现有 `TrackedTaskState`。
2. `alert cluster draft` 与 `raw alert event`、`follow_up_items` 的实现边界应怎样处理。
3. 哪些代码文件值得先改，哪些文件暂时不要碰。
4. 当前 task detail API、repository 和持久化基线怎样承接这次改动。
5. ingestion、cluster query、smoke 和兼容性第一刀应先做到什么程度。

本文件不是 event-ingestion 已经实现的声明，也不是 alert cluster projection 设计说明。它只服务于告警 task detail 第一刀代码收口。

## 2. 实施原则

告警 cluster draft 的 implementation cut 继续坚持以下原则：

- 先把 `alert_cluster_drafts` 做成 task-scoped read model，不先做独立 cluster query API。
- 先扩 `tasks/{task_id}`，不先开放 raw event ingestion 入口。
- `raw alert event` 不直接进入 `TrackedTaskState` 顶层结构。
- `alert_cluster_draft` 不直接等于 `follow-up item`，不要把后续执行字段直接塞进 cluster draft。
- 即使 `Scada / Andon` mock connector 代码还没落，cluster draft 也应能以空数组安全落地。
- Phase A 只解决“任务详情能稳定表达异常簇和分诊摘要”，不解决“所有原始信号如何实时进入平台”。

## 3. 当前代码基线快照

| 代码锚点 | 当前状态 | 对 implementation cut 的意义 |
| --- | --- | --- |
| `TrackedTaskState` | 当前是统一任务详情载体 | `alert_cluster_drafts` 最适合直接挂进这里 |
| `GET /api/v1/tasks/{task_id}` | 直接返回 `TrackedTaskState` | 不需要新 endpoint 就能把 cluster draft 暴露给调用方 |
| `TaskRepository` | 读写整个 `TrackedTaskState` | Phase A 不必修改 repository trait |
| file repository | 持久化整包 JSON | 新增字段可通过 `serde(default)` 兼容旧任务 |
| SQLite repository | `payload_json` 持久化整包状态 | 不需要增加 `alert_clusters` 表 |
| `requested_record_kinds()` | 当前还没有告警源的读取语义 | schema cut 不应被 connector 缺口阻塞 |

## 4. 推荐的最小 schema cut

### 4.1 先加在 `TrackedTaskState` 上的字段

Phase A 最小改动建议新增两个字段：

- `alert_cluster_drafts`
- `alert_triage_summary`

推荐方向：

```rust
#[serde(default)]
pub alert_cluster_drafts: Vec<AlertClusterDraftView>,
#[serde(default)]
pub alert_triage_summary: AlertTriageSummary,
```

这样做有 4 个直接好处：

1. `GET /api/v1/tasks/{task_id}` 自动把 cluster draft 带给调用方。
2. in-memory / file / SQLite repository 都能自动跟随持久化。
3. 旧任务 JSON 缺少字段时可以依赖默认值安全回读。
4. 告警 task detail 可以先稳定，再决定后续 cluster query 和 ingestion 资源。

### 4.2 为什么这里不推荐 `raw_alert_events`

这一刀不建议直接在 `TrackedTaskState` 顶层加：

- `raw_alert_events`
- `ingestion_decisions`
- `cluster_projection_state`

原因很直接：

- 任务详情更需要的是“已经归并后的分诊可读对象”，不是事件原始流水。
- 原始事件更适合作为 evidence 或后续 ingestion/projection 内部输入。
- 如果顶层直接接 raw events，task detail 很容易重新被噪音淹没。

### 4.3 为什么这一步不建议先加更多容器

这一刀不建议一上来同时引入：

- `alert_clusters`
- `triage_routes`
- `alert_ingestion_runs`
- `cluster_query_projection`

原因很直接：

- `alert_cluster_drafts + alert_triage_summary` 已足够支撑 task detail。
- 额外对象会让 Phase A 从“task detail cut”膨胀成“事件系统设计”。
- ingress、projection、聚合查询都应放到下一阶段。

## 5. 推荐的最小对象形态

### 5.1 `AlertClusterDraftView`

第一版建议把它定义成 task detail view，而不是正式 domain aggregate。

更合理的最小字段如下：

| 字段 | 作用 | Phase A 建议 |
| --- | --- | --- |
| `cluster_id` | 稳定标识一个异常簇 | 必需 |
| `cluster_status` | 如 `open / monitoring / stabilized / escalated / closed` | 必需 |
| `source_system` | 当前主来源系统，如 `scada / andon / incident_log` | 建议 |
| `equipment_id` | 异常关联设备 | 可选 |
| `line_id` | 异常关联产线 | 可选 |
| `severity_band` | 严重度分层 | 必需 |
| `source_event_refs` | 归并进该簇的事件引用 | 建议，默认空数组 |
| `window_start` | 聚类窗口起点 | 必需 |
| `window_end` | 聚类窗口终点 | 必需 |
| `triage_label` | 当前分诊标签 | 建议 |
| `recommended_owner_role` | 建议路由角色 | 建议 |
| `escalation_candidate` | 是否建议升级 | 必需 |
| `rationale` | 简短分诊理由 | 可选 |
| `created_at / updated_at` | 排序、回放和 projection 重建基础 | 必需 |

### 5.2 `AlertTriageSummary`

第一版 summary 建议至少包含：

- `total_clusters`
- `open_clusters`
- `high_priority_clusters`
- `escalation_candidate_count`
- `last_clustered_at`

### 5.3 与 evidence、follow-up 的边界

这一层必须明确两个现实约束：

- `alert_cluster_drafts` 可以先于真实 ingestion 管线落地
- `alert_cluster_drafts` 可以先于非空 `follow_up_items` 落地

更合理的关系是：

- raw alert 细节继续以 evidence 或 mock connector payload 存在
- cluster draft 只表达归并与分诊后的 task detail 视图
- follow-up item 在后续阶段再从 cluster draft 派生

这样可以避免告警对象和执行对象混在一起。

## 6. 类型放置建议

第一版更建议把这些类型放在 `fa-core` 的 task-detail / orchestrator read-model 层，而不是急着沉到 `fa-domain`。

理由：

- cluster draft 当前只是 task detail 的读层输出
- 当前还没有 cluster 状态机、ingestion adapter 和专属审计事件代码
- 如果现在把它定义成稳定 domain object，后续容易被误解为“事件主线写路径已经冻结”

## 7. 代码触点建议

### 7.1 `crates/fa-core/src/orchestrator.rs`

这是第一优先级触点。

建议动作：

1. 在 `TrackedTaskState` 上增加 `alert_cluster_drafts` 和 `alert_triage_summary`
2. 新增最小 view struct 与 `Default` 实现
3. 在 `intake_task_with_correlation()` 中初始化默认值
4. 保持 `approve / execute / resubmit / complete / fail` 不承担 cluster 状态推进逻辑

Phase A 最重要的结果不是“告警已经可实时接入”，而是“任务详情 contract 正式拥有 cluster draft 结构”。

### 7.2 `apps/fa-server/src/main.rs`

这里原则上不需要新增路由。

建议动作：

- 保持 `GET /api/v1/tasks/{task_id}` 继续直接返回 `TrackedTaskState`
- 只在 API 契约说明和验证文档中补充 `alert_cluster_drafts` 字段说明

### 7.3 `crates/fa-core/src/repository.rs`

repository 接口层不建议改签名。

建议动作：

- 维持 `TaskRepository` 现有 `create / get / save` 契约
- 为 in-memory / file / SQLite round-trip 补测试
- 继续复用 file JSON 与 SQLite `payload_json`，不引入告警专属表

### 7.4 smoke、测试与 QA 文档

第一版最值得补的验证有 4 类：

1. `serde` 兼容性：旧任务 JSON 缺少 cluster 字段时仍能回读
2. repository round-trip：cluster 字段能在 in-memory / file / SQLite 模式下保存与回读
3. API contract：`GET /api/v1/tasks/{task_id}` 返回 `alert_cluster_drafts` 与 `alert_triage_summary`
4. smoke 断言：sandbox-safe smoke 至少验证这两个字段存在且默认可解析

## 8. 推荐的 Phase A 实施顺序

### Phase A1. Cluster Schema and Persistence Cut

目标：

- 先让告警任务详情拥有正式的 cluster draft 字段

建议交付：

1. 增加 `AlertClusterDraftView` 与 `AlertTriageSummary`
2. 扩展 `TrackedTaskState`
3. 更新 repository round-trip 测试
4. 更新 API / smoke / QA 文档断言

退出标准：

- 新老任务都能稳定回读 `alert_cluster_drafts` 相关字段

### Phase A2. Seeded Draft Cluster Population

目标：

- 让至少一个告警任务示例能返回非空 cluster draft

建议交付：

1. 增加受控 helper，从 mock alert evidence / connector payload 派生 cluster draft
2. 保持所有对象为 read-only / draft 输出
3. 暂不增加 ingestion endpoint

退出标准：

- 至少一类告警任务在 `tasks/{task_id}` 中能显示非空 `alert_cluster_drafts`

## 9. 本阶段明确不做的事

这一刀应明确延后以下事项：

- 不新增 `POST /api/v1/alert-events/ingest`
- 不新增 `GET /api/v1/alert-clusters`
- 不把 raw alert 事件直接挂进 task detail 顶层
- 不新增 cluster 专属数据库表
- 不把 cluster draft 状态接进 task lifecycle
- 不在同一批改动里顺手引入 `Scada / Andon / incident_log` connector 代码

## 10. 推荐的 API 形态示例

第一版告警任务详情更适合长这样：

```json
{
  "correlation_id": "demo-alert-001",
  "planned_task": { "...": "..." },
  "context_reads": [],
  "evidence": [],
  "follow_up_items": [],
  "follow_up_summary": {
    "total_items": 0,
    "open_items": 0,
    "blocked_items": 0,
    "overdue_items": 0,
    "escalated_items": 0,
    "last_evaluated_at": null
  },
  "alert_cluster_drafts": [
    {
      "cluster_id": "ac_001",
      "cluster_status": "open",
      "source_system": "andon",
      "equipment_id": "eq_pack_04",
      "line_id": "line_pack_a",
      "severity_band": "high",
      "source_event_refs": ["andon:evt-101", "andon:evt-102"],
      "window_start": "2026-03-12T09:12:00Z",
      "window_end": "2026-03-12T09:18:00Z",
      "triage_label": "repeat_temperature_alarm",
      "recommended_owner_role": "production_supervisor",
      "escalation_candidate": true,
      "rationale": "Repeated alarm burst within short window on same station.",
      "created_at": "2026-03-12T09:18:00Z",
      "updated_at": "2026-03-12T09:18:00Z"
    }
  ],
  "alert_triage_summary": {
    "total_clusters": 1,
    "open_clusters": 1,
    "high_priority_clusters": 1,
    "escalation_candidate_count": 1,
    "last_clustered_at": "2026-03-12T09:18:00Z"
  }
}
```

重点不在第一版就把告警事件系统做完，而在：

- key 名稳定
- 非告警任务默认兼容
- 与 evidence、follow-up 并列但不互相替代
- 后续 ingestion 和 cluster query 可以沿同一 contract 继续扩展

## 11. 结论

`alert_cluster_drafts` 最合理的第一刀，不是直接做事件入口或 cluster backlog，而是先进入现有 `TrackedTaskState` 和 `tasks/{task_id}`。

这一步实现成本低、兼容性好、存储改动小，而且能先把“哪些信号已经形成需要分诊的异常簇”变成正式 task detail 语义。

下一步最合理的动作有两个：

1. 把 `follow_up_items`、`handoff_receipt`、`alert_cluster_drafts` 的 schema cut 从说明推进到代码实现。
2. 评估把 mock `QMS` baseline 从设计说明推进到默认 registry 和 read plan 代码实现的切口。
