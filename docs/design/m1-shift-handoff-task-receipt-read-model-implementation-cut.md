# M1 Shift Handoff Task Receipt Read Model Implementation Cut

## 1. 文档目的

本文件把 `shift handoff receipt / acknowledgement` 从方向说明继续推进到“最小可实施代码切口”。

它重点回答 5 个问题：

1. 最小 `handoff_receipt` 应先怎样进入现有 `TrackedTaskState`。
2. `handoff_receipt` 与 `follow_up_items` 的实现依赖应怎样处理，避免互相阻塞。
3. 哪些代码文件值得先改，哪些文件暂时不要碰。
4. 当前 task detail API、repository 和持久化基线怎样承接这次改动。
5. smoke、测试和兼容性第一刀应先做到什么程度。

本文件不是 receipt 已经实现的声明，也不是 acknowledgement action API 设计说明。它只服务于交接 task detail 第一刀代码收口。

## 2. 实施原则

交接 receipt 的 implementation cut 继续坚持以下原则：

- 先把 `handoff_receipt` 做成 task-scoped read model，不先做 acknowledgement write API。
- 先扩 `tasks/{task_id}`，不先扩跨班次 queue / projection。
- `handoff_receipt` 不是 `approval`，不要复用审批状态和审批接口。
- `handoff_receipt` 不是 `follow-up accepted owner`，不要把 item owner 接手逻辑塞进 receipt 对象。
- 即使 `follow_up_items` 代码还没落，`handoff_receipt` 也应能以空 `follow_up_item_ids` 和零计数安全落地。
- Phase A 只解决“任务详情能稳定表达交接包是否被接住”，不解决“谁点击确认”和“全厂交接 backlog”。

## 3. 当前代码基线快照

| 代码锚点 | 当前状态 | 对 implementation cut 的意义 |
| --- | --- | --- |
| `TrackedTaskState` | 当前是统一任务详情载体 | `handoff_receipt` 最适合直接挂进这里 |
| `GET /api/v1/tasks/{task_id}` | 直接返回 `TrackedTaskState` | 不需要新 endpoint 就能把 receipt 暴露给调用方 |
| `TaskRepository` | 读写整个 `TrackedTaskState` | Phase A 不必修改 repository trait |
| file repository | 持久化整包 JSON | 新增字段可通过 `serde(default)` 兼容旧任务 |
| SQLite repository | `payload_json` 持久化整包状态 | 不需要增加 `handoff_receipts` 表 |
| `follow_up_items` implementation cut note | 已明确 follow-up 会进入 task detail | receipt 可并列扩展，而不需要等待 follow-up queue 实现 |

## 4. 推荐的最小 schema cut

### 4.1 先加在 `TrackedTaskState` 上的字段

Phase A 最小改动建议新增两个字段：

- `handoff_receipt`
- `handoff_receipt_summary`

推荐方向：

```rust
#[serde(default)]
pub handoff_receipt: Option<HandoffReceiptView>,
#[serde(default)]
pub handoff_receipt_summary: HandoffReceiptSummary,
```

这样做有 4 个直接好处：

1. `GET /api/v1/tasks/{task_id}` 自动把 receipt 带给调用方。
2. in-memory / file / SQLite repository 都能无感跟随持久化。
3. 旧任务 JSON 缺少字段时可以依赖默认值安全回读。
4. receipt 与 `follow_up_items` 可以在同一 task detail contract 中并列存在。

### 4.2 为什么这里推荐 `Option<HandoffReceiptView>`

receipt 和 follow-up 不一样，它不是所有任务都天然需要的通用字段。

因此第一版更适合：

- `handoff_receipt: None` 表示这不是交接任务，或当前还没有 receipt draft
- `handoff_receipt_summary` 默认零值或空值，保证调用方 contract 稳定

这样可以避免把“非交接任务没有 receipt”误表达成“一条空 receipt”。

### 4.3 为什么这一步不建议先加更多交接对象

这一刀不建议一上来同时引入：

- `handoff_package`
- `handoff_ack_events`
- `handoff_receipt_queue_projection`
- `handoff_exception_items`

原因很直接：

- `handoff_receipt + handoff_receipt_summary` 已足够支撑 task detail。
- 额外对象会让 Phase A 从“任务详情 cut”膨胀成“交接子系统设计”。
- acknowledgement action、跨班次查询、异常项拆分都应放到下一阶段。

## 5. 推荐的最小对象形态

### 5.1 `HandoffReceiptView`

第一版建议把它定义成 task detail view，而不是正式 domain aggregate。

更合理的最小字段如下：

| 字段 | 作用 | Phase A 建议 |
| --- | --- | --- |
| `id` | 稳定标识一次交接回执 | 必需 |
| `handoff_task_id` | 关联交接任务 | 必需 |
| `shift_id` | 交接窗口或班次标识 | 必需 |
| `sending_actor` | 发出交接包的人 | 必需 |
| `receiving_role` | 默认接收角色 | 必需 |
| `receiving_actor` | 实际接收人 | 可选 |
| `published_at` | 交接包发出时间 | 必需 |
| `required_ack_by` | 要求接收确认时间 | 建议 |
| `status` | 如 `draft / published / acknowledged / acknowledged_with_exceptions / escalated / expired` | 必需 |
| `follow_up_item_ids` | 本次交接包覆盖的遗留事项 ID 列表 | 建议，默认空数组 |
| `exception_note` | 接收方异议或信息缺口说明 | 可选 |
| `acknowledged_at` | 正式确认接收时间 | 可选 |
| `escalation_state` | 是否已升级 | 建议 |
| `created_at / updated_at` | 排序、回放和审计重建基础 | 必需 |

### 5.2 `HandoffReceiptSummary`

第一版 summary 建议至少包含：

- `status`
- `published_at`
- `required_ack_by`
- `acknowledged_at`
- `covered_follow_up_count`
- `unaccepted_follow_up_count`
- `exception_flag`

### 5.3 与 `follow_up_items` 的依赖处理

这一层必须明确一个现实约束：

- `handoff_receipt` 可以先于非空 `follow_up_items` 落地
- `follow_up_item_ids` 在第一版允许为空数组
- `covered_follow_up_count` 与 `unaccepted_follow_up_count` 在第一版允许为 `0`

这意味着交接 receipt 的 schema cut 不必等待 follow-up code cut 完成。

更合理的推进关系是：

1. 先让 task detail 能稳定返回 `handoff_receipt`
2. 再在后续阶段把 receipt 与实际 `follow_up_items` 关联起来

这能避免两个高频对象互相卡死。

## 6. 类型放置建议

第一版更建议把这些类型放在 `fa-core` 的 task-detail / orchestrator read-model 层，而不是急着沉到 `fa-domain`。

理由：

- receipt 当前只是 task detail 的读层输出
- 当前还没有 acknowledgement command、receipt 状态机和专属审计事件代码
- 如果现在把它定义成稳定 domain object，后续容易被误解为“交接闭环写路径已经冻结”

## 7. 代码触点建议

### 7.1 `crates/fa-core/src/orchestrator.rs`

这是第一优先级触点。

建议动作：

1. 在 `TrackedTaskState` 上增加 `handoff_receipt` 和 `handoff_receipt_summary`
2. 新增最小 view struct 与 `Default` 实现
3. 在 `intake_task_with_correlation()` 中初始化默认值
4. 保持 `approve / execute / resubmit / complete / fail` 不承担 receipt 状态推进逻辑

Phase A 最重要的结果不是“交接已经可确认”，而是“任务详情 contract 正式拥有 receipt 结构”。

### 7.2 `apps/fa-server/src/main.rs`

这里原则上不需要新增路由。

建议动作：

- 保持 `GET /api/v1/tasks/{task_id}` 继续直接返回 `TrackedTaskState`
- 只在 API 契约说明和验证文档中补充 `handoff_receipt` 字段说明

### 7.3 `crates/fa-core/src/repository.rs`

repository 接口层不建议改签名。

建议动作：

- 维持 `TaskRepository` 现有 `create / get / save` 契约
- 为 in-memory / file / SQLite round-trip 补测试
- 继续复用 file JSON 与 SQLite `payload_json`，不引入交接专属表

### 7.4 smoke、测试与 QA 文档

第一版最值得补的验证有 4 类：

1. `serde` 兼容性：旧任务 JSON 缺少 receipt 字段时仍能回读
2. repository round-trip：receipt 字段能在 in-memory / file / SQLite 模式下保存与回读
3. API contract：`GET /api/v1/tasks/{task_id}` 返回 `handoff_receipt` 与 `handoff_receipt_summary`
4. smoke 断言：sandbox-safe smoke 至少验证这两个字段存在且默认可解析

## 8. 推荐的 Phase A 实施顺序

### Phase A1. Receipt Schema and Persistence Cut

目标：

- 先让交接任务详情拥有正式的 receipt 字段

建议交付：

1. 增加 `HandoffReceiptView` 与 `HandoffReceiptSummary`
2. 扩展 `TrackedTaskState`
3. 更新 repository round-trip 测试
4. 更新 API / smoke / QA 文档断言

退出标准：

- 新老任务都能稳定回读 `handoff_receipt` 相关字段

### Phase A2. Seeded Draft Receipt Population

目标：

- 让至少一个交接任务示例能返回非空 receipt draft

建议交付：

1. 增加受控 helper，从交接任务 request / evidence 生成一条 draft receipt
2. 保持所有状态为 read-only / draft 输出
3. 暂不增加 acknowledgement action

退出标准：

- 至少一类交接任务在 `tasks/{task_id}` 中能显示非空 `handoff_receipt`

## 9. 本阶段明确不做的事

这一刀应明确延后以下事项：

- 不新增 `POST /api/v1/handoff-receipts/{id}/acknowledge`
- 不把 receipt 状态接进审批状态机
- 不新增跨班次 `handoff_receipts` 聚合查询 API
- 不新增 receipt 专属数据库表
- 不把 `follow_up_items` 的 accepted owner 逻辑塞进 receipt
- 不在同一批改动里顺手引入 `shift_log / incident_log` connector 代码

## 10. 推荐的 API 形态示例

第一版交接任务详情更适合长这样：

```json
{
  "correlation_id": "demo-handoff-001",
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
  "handoff_receipt": {
    "id": "hr_001",
    "handoff_task_id": "72c8f5d0-0f08-4e0c-a8c4-1d4dc51a25f0",
    "shift_id": "shift_b_2026_03_12",
    "sending_actor": {
      "id": "worker_1001",
      "display_name": "Liu Supervisor",
      "role": "Production Supervisor"
    },
    "receiving_role": "incoming_shift_supervisor",
    "receiving_actor": null,
    "published_at": "2026-03-12T13:30:00Z",
    "required_ack_by": "2026-03-12T14:00:00Z",
    "status": "published",
    "follow_up_item_ids": [],
    "exception_note": null,
    "acknowledged_at": null,
    "escalation_state": "none",
    "created_at": "2026-03-12T13:30:00Z",
    "updated_at": "2026-03-12T13:30:00Z"
  },
  "handoff_receipt_summary": {
    "status": "published",
    "published_at": "2026-03-12T13:30:00Z",
    "required_ack_by": "2026-03-12T14:00:00Z",
    "acknowledged_at": null,
    "covered_follow_up_count": 0,
    "unaccepted_follow_up_count": 0,
    "exception_flag": false
  }
}
```

重点不在第一版就把交接闭环做完，而在：

- key 名稳定
- 非交接任务默认兼容
- 与 `follow_up_items` 并列但不互相阻塞
- 后续 acknowledgement action 和 queue projection 可以沿同一 contract 继续扩展

## 11. 结论

`handoff_receipt` 最合理的第一刀，不是直接做确认动作或跨班次队列，而是先进入现有 `TrackedTaskState` 和 `tasks/{task_id}`。

这一步实现成本低、兼容性好、存储改动小，而且能先把“交接包是否被接住”变成正式 task detail 语义。

下一步最合理的动作有两个：

1. 继续为 `alert_cluster_drafts` 输出同类型 implementation cut note。
2. 把 `follow_up_items` 的 task detail schema cut 从说明推进到代码实现。
