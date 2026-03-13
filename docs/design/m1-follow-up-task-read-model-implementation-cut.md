# M1 Follow-up Task Read Model Implementation Cut

## 1. 文档目的

本文件把 `follow-up / SLA` 从 read model 方向说明继续推进到“最小可实施代码切口”。

它重点回答 5 个问题：

1. 最小 `follow_up_items` 应先怎样进入现有 `TrackedTaskState`。
2. 这一步最值得先触达哪些代码文件，哪些文件暂时不要碰。
3. 现有 task detail API、repository 和持久化基线怎样承接这次改动。
4. smoke、测试和兼容性应先做到什么程度。
5. 哪些能力必须明确延后，避免 Phase A 过早膨胀成新的任务系统。

本文件不是 `follow_up_items` 已经实现的声明，也不是跨任务 queue API 设计说明。它只服务于第一刀代码收口。

## 2. 实施原则

实现切口继续坚持以下原则：

- 先扩 `tasks/{task_id}` 的 task-scoped read model，不先做跨任务 projection。
- 先做 read-only / draft 输出，不先引入 item 写接口。
- 先让 schema、序列化和持久化稳定，再谈自动生成更多 follow-up items。
- `PlannedStep.owner` 不是正式 `follow-up owner`，不要复用成 item 责任人本体。
- `handoff_receipt`、`alert_cluster_drafts`、`quality containment` 仍保留为场景专属对象，不吞进这次 cut。
- Phase A 只解决“任务详情里能稳定看见 follow-up 结构”，不解决“谁来维护全厂 backlog”。

## 3. 当前代码基线快照

| 代码锚点 | 当前状态 | 对 implementation cut 的意义 |
| --- | --- | --- |
| `TrackedTaskState` | 当前只包含 `correlation_id / planned_task / context_reads / evidence` | 是新增 `follow_up_items` 的最小挂载点 |
| `GET /api/v1/tasks/{task_id}` | 直接返回 `TrackedTaskState` | 不需要新 endpoint，就能把 read model 暴露给调用方 |
| `TaskRepository` | `create / get / save` 读写整个 `TrackedTaskState` | Phase A 可不改 repository 接口 |
| file repository | 直接把整个状态写成 JSON 文件 | 新字段只要带 `serde(default)` 即可兼容旧文件 |
| SQLite repository | `tasks.payload_json` 存整包 JSON | 新字段不要求表结构迁移 |
| smoke / manual validation | 已围绕 `tasks/intake -> tasks/{task_id}` 主链组织 | 只需补 task detail 断言，不必开第二条演示路径 |

## 4. 推荐的最小 schema cut

### 4.1 先加在 `TrackedTaskState` 上的字段

Phase A 最小改动建议只新增两个字段：

- `follow_up_items`
- `follow_up_summary`

推荐方向：

```rust
#[serde(default)]
pub follow_up_items: Vec<FollowUpItemView>,
#[serde(default)]
pub follow_up_summary: FollowUpSummary,
```

这样做有 4 个直接好处：

1. `GET /api/v1/tasks/{task_id}` 自动返回新字段。
2. in-memory / file / SQLite repository 都能自动跟着持久化。
3. 旧 JSON 缺少字段时可以依赖 `serde(default)` 回填空值。
4. 后续 `handoff_receipt`、`alert_cluster_drafts` 也能沿同一条 task detail 主链扩展。

### 4.2 为什么这一步不建议先加更多容器

这一刀不建议一上来同时引入：

- `follow_up_view`
- `sla_summary`
- `follow_up_queue_projection`
- `follow_up_audit_timeline`

原因很直接：

- `follow_up_items + follow_up_summary` 已足够支撑任务详情。
- 额外容器会把 Phase A 从“task detail schema cut”扩大成“新读模型体系重构”。
- 真正需要跨任务查询时，再单独引入 projection 资源更清晰。

## 5. 推荐的最小对象形态

### 5.1 `FollowUpItemView`

第一版建议把它定义成 task detail view，而不是正式 domain aggregate。

更合理的最小字段如下：

| 字段 | 作用 | Phase A 建议 |
| --- | --- | --- |
| `id` | 稳定标识单条 item | 必需 |
| `title` | 简短待办标题 | 必需 |
| `summary` | 给前端和调用方的简要说明 | 建议 |
| `source_kind` | 如 `handoff / alert_cluster / quality_deviation / anomaly` | 必需 |
| `source_refs` | 引用 evidence、connector record、note 或 cluster | 建议 |
| `status` | 如 `draft / accepted / in_progress / blocked / completed / escalated` | 必需 |
| `recommended_owner_role` | 当前建议的责任角色 | 建议 |
| `accepted_owner_id` | 是否已有明确接手人 | 建议 |
| `due_at` | item 级到期时间 | 建议 |
| `sla_status` | `on_track / due_soon / overdue / escalation_required` | 必需 |
| `blocked_reason` | 阻塞说明 | 可选 |
| `created_at / updated_at` | item 级排序和回放基线 | 必需 |

这里刻意不要求第一版就引入一组复杂的嵌套对象。

原因是本阶段目标不是一次性完成正式 `follow-up` 领域建模，而是先让 task detail 具备稳定、可兼容、可落盘的结构化输出。

### 5.2 `FollowUpSummary`

第一版 summary 建议至少包含：

- `total_items`
- `open_items`
- `blocked_items`
- `overdue_items`
- `escalated_items`
- `last_evaluated_at`

它的价值是让任务详情调用方不用每次手动遍历 item 列表，先能拿到一个轻量总览。

### 5.3 类型放置建议

第一版更建议把这些类型放在 `fa-core` 的 task-detail / orchestrator read-model 层，而不是急着下沉到 `fa-domain`。

理由：

- 这些对象当前只是 `tasks/{task_id}` 的读层输出。
- 现在还没有 item 级写 API、状态机和专属审计事件。
- 如果现在就把它们定义成稳定 domain object，后续容易被错误承诺为“正式业务对象已冻结”。

## 6. 代码触点建议

### 6.1 `crates/fa-core/src/orchestrator.rs`

这是第一优先级触点。

建议动作：

1. 在 `TrackedTaskState` 上增加 `follow_up_items` 和 `follow_up_summary`。
2. 新增最小 view struct 和 `Default` 实现。
3. 在 `intake_task_with_correlation()` 中初始化默认空值。
4. 保持 `approve / execute / resubmit / complete / fail` 只做状态透传，不在这一刀引入 item 级变更逻辑。

Phase A 最重要的结果不是“自动生成很多 item”，而是“任务详情 contract 稳定成立”。

### 6.2 `apps/fa-server/src/main.rs`

这里原则上不需要新增 endpoint。

建议动作：

- 保持 `GET /api/v1/tasks/{task_id}` 原样返回 `TrackedTaskState`。
- 只在 API 契约和验证文档中补充新字段说明。

这能保证实现切口留在核心状态对象，而不是把改动扩散成新的路由设计。

### 6.3 `crates/fa-core/src/repository.rs`

repository 接口层不建议改签名。

建议动作：

- 维持 `TaskRepository` 现有 `create / get / save` 契约不变。
- 为 in-memory / file / SQLite round-trip 补测试，确认新字段能稳定落盘与回读。
- 继续使用 SQLite `payload_json` 直存整包状态，不在这一刀引入新表。

### 6.4 smoke、测试与 QA 文档

第一版最值得补的验证有 4 类：

1. `serde` 兼容性：旧任务 JSON 缺少新字段时仍能回读成功。
2. repository round-trip：新字段在 in-memory / file / SQLite 模式下都能保存并回读。
3. API contract：`GET /api/v1/tasks/{task_id}` 返回 `follow_up_items` 和 `follow_up_summary`。
4. smoke 断言：sandbox-safe smoke 至少验证这两个字段存在且默认可解析。

## 7. 推荐的 Phase A 实施顺序

### Phase A1. Schema and Persistence Cut

目标：

- 先让 `tasks/{task_id}` 正式拥有 `follow_up_items` 和 `follow_up_summary`

建议交付：

1. 增加 view struct 与默认值。
2. 扩展 `TrackedTaskState`。
3. 更新 repository round-trip 测试。
4. 更新 API / smoke / QA 文档断言。

退出标准：

- 新老任务都能通过 `tasks/{task_id}` 稳定返回 follow-up task detail 字段。

### Phase A2. Seeded Draft Population

目标：

- 让至少一个高频场景能产出非空 follow-up draft

建议交付：

1. 增加受控 helper，从现有 `planned_task / evidence` 派生少量 draft items。
2. 先从单场景示例开始，不做全平台自动推断。
3. 继续保持 read-only，不补 item write API。

退出标准：

- 至少一类任务在 `tasks/{task_id}` 中能展示非空 `follow_up_items`。

## 8. 本阶段明确不做的事

这一刀应明确延后以下事项：

- 不新增 `GET /api/v1/follow-up-items`
- 不新增 follow-up item 写接口或指派接口
- 不新增 item 级审计事件体系
- 不把 `PlannedStep.owner` 改造成 item owner
- 不把 `handoff_receipt` 或 `alert_cluster_drafts` 混进这次 schema cut
- 不在同一批改动里顺手引入 `mock QMS` connector 代码

## 9. 推荐的 API 形态示例

第一版 task detail 更适合长这样：

```json
{
  "correlation_id": "demo-intake-001",
  "planned_task": { "...": "..." },
  "context_reads": [],
  "evidence": [],
  "follow_up_items": [
    {
      "id": "fu_001",
      "title": "Confirm coolant loop inspection result",
      "summary": "Need maintenance confirmation before next shift.",
      "source_kind": "anomaly",
      "source_refs": ["mes:wo-1001", "cmms:inspection-88"],
      "status": "draft",
      "recommended_owner_role": "maintenance_supervisor",
      "accepted_owner_id": null,
      "due_at": "2026-03-12T18:00:00Z",
      "sla_status": "due_soon",
      "blocked_reason": null,
      "created_at": "2026-03-12T09:10:00Z",
      "updated_at": "2026-03-12T09:10:00Z"
    }
  ],
  "follow_up_summary": {
    "total_items": 1,
    "open_items": 1,
    "blocked_items": 0,
    "overdue_items": 0,
    "escalated_items": 0,
    "last_evaluated_at": "2026-03-12T09:10:00Z"
  }
}
```

重点不在字段一次到顶，而在：

- key 名稳定
- 默认值稳定
- 旧任务兼容
- 后续 `handoff`、`alert`、`quality` 都能沿同一条主链继续扩展

## 10. 结论

`follow_up_items` 最合理的第一刀，不是新开一套 backlog 系统，而是先进入现有 `TrackedTaskState` 和 `tasks/{task_id}`。

这一步的实现成本低、兼容性好、存储改动小，而且能直接为后续 `handoff_receipt`、`alert_cluster_drafts` 和 `quality` follow-up 打开统一任务详情入口。

下一步最合理的动作有两个：

1. 继续为 `shift handoff receipt` 输出同类型 implementation cut note。
2. 继续为 `alert triage cluster draft` 输出同类型 implementation cut note。
