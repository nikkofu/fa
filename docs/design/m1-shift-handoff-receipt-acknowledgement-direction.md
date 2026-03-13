# M1 Shift Handoff Receipt and Acknowledgement Direction

## 1. 文档目的

本文件把 `班次交接摘要与待办提取` 继续从 alignment checklist 推进到 `receipt / acknowledgement` 专属对象方向。

它重点回答 5 个问题：

1. 为什么 `handoff receipt` 不能直接并入通用 `follow-up item` 状态。
2. 交接场景里“确认发送”“接收确认”“遗留事项接手”三类语义应怎样拆分。
3. 当前任务主链、evidence、governance 和审计能先承接什么。
4. 交接 receipt 应先怎样进入 task detail，再怎样进入跨班次 query。
5. 后续实现应按什么顺序推进，而不是把 receipt 继续停留在口头语义层。

本文件不是代码实现说明，也不是“交接 receipt 已经存在”的声明。它是后续交接专属 read model、确认动作和跨班次查询的方向说明。

## 2. 方向原则

交接 receipt 方向继续坚持以下原则：

- `receipt / acknowledgement` 不是 `approval`，不能硬套审批状态机。
- `receipt` 不是 `follow-up accepted owner`，两者必须拆开。
- 先做 task-scoped receipt read model，再做跨班次 receipt queue。
- 先支持“已发出、已接收、已确认、有异议”这些协同状态，再考虑更细的 item-level 回执。
- 高风险遗留事项是否升级，仍回到 governed workflow，而不是由 receipt 状态直接裁决。
- receipt 对象应描述“交接包是否被接住”，而不是替代 follow-up item 本身。

## 3. 为什么通用 follow-up 还不够

`follow-up item` 解决的是：

- 有哪些遗留事项
- 谁建议接手
- 谁已经接受
- 什么时候该完成
- 是否逾期或阻塞

但 `handoff receipt` 解决的是另一类问题：

- 本次交接是否已经发给下一班
- 下一班是否已经看到和接收这份交接包
- 是否存在接收时的异议、信息缺口或需要主管介入的项
- 交接包本身是否超时未接收

因此，交接 receipt 与 follow-up 的关系应是：

- receipt 作用于“交接包”或“交接任务整体”
- follow-up 作用于“交接包里的具体遗留事项”
- 两者有关联，但不是同一个状态机

## 4. 当前平台基线快照

| 对象 | 当前状态 | 对 receipt 的意义 |
| --- | --- | --- |
| `TrackedTaskState` | `tasks/{task_id}` 已是统一任务详情入口 | receipt 最适合先挂在任务详情上 |
| `follow-up / SLA` 方向文档 | 已明确 `follow_up_items` 应进入 task detail | 交接 receipt 可与 follow-up 并列，而不必混入 item 状态 |
| `WorkflowGovernance` | 已能表达责任矩阵和 fallback actions | 可承接发送方、接收方和升级边界，但不是 receipt 本体 |
| `TaskEvidence` | 能表达交接线索和摘要来源 | 可引用交接包来源，但不能表达 receipt 状态变化 |
| audit 主链 | 任务级审计和过滤已存在 | 适合回放 `published / acknowledged / escalated` 事件 |
| API 主链 | `tasks/{task_id}`、`evidence`、`governance`、`audit-events` 已存在 | Phase A 不必先新增交接专属 endpoint |

## 5. 交接语义拆分方向

### 5.1 发送确认

发送确认回答的是：

- 交接摘要是否已经整理完成
- 是否已经发给接收班次
- 由谁发出
- 发出时包含哪些 follow-up items

这更接近 `published` 或 `sent` 语义。

### 5.2 接收确认

接收确认回答的是：

- 接收班次是否已经看到交接包
- 是否确认已接住本次交接
- 是否标记存在信息缺口或接收异议

这就是 `receipt / acknowledgement` 本体。

### 5.3 遗留事项接手

遗留事项接手回答的是：

- 哪条 follow-up 由谁接
- 是建议 owner 还是 accepted owner
- 哪些项尚未明确 owner

这仍属于通用 `follow-up item` 范畴。

### 5.4 三者之间的关键边界

最需要避免的误区有 3 个：

1. 不要把“交接包已被接收”误写成“所有遗留事项都已有 accepted owner”。
2. 不要把“接收方提出异议”误写成“审批拒绝”。
3. 不要把“高风险事项已升级”误写成“交接 receipt 已完成”。

## 6. 推荐的对象方向

### 6.1 Task-scoped `handoff_receipt` 对象

第一版最适合先进入 `tasks/{task_id}` 的对象是 `handoff_receipt`。

建议最小字段如下：

| 字段 | 作用 | 建议 |
| --- | --- | --- |
| `id` | 稳定标识一次交接回执 | 必需 |
| `handoff_task_id` | 关联交接任务 | 必需 |
| `shift_id` | 交接所属班次或交接窗口标识 | 必需 |
| `sending_actor` | 发出交接的人 | 必需 |
| `receiving_role` | 默认接收角色 | 必需 |
| `receiving_actor` | 实际接收人，可为空 | Phase A 可选 |
| `published_at` | 交接包发出时间 | 必需 |
| `required_ack_by` | 要求确认接收的时间窗口 | 建议 |
| `status` | 回执状态 | 必需 |
| `follow_up_item_ids` | 这次交接包覆盖的遗留事项 | 必需 |
| `exception_note` | 接收时的缺口说明或异议说明 | 建议 |
| `acknowledged_at` | 正式确认接收时间 | 建议 |
| `escalation_state` | 是否已升级给主管或值班 | 建议 |

### 6.2 推荐的最小状态集合

第一版 receipt 状态不应太复杂，建议先支持：

- `draft`
- `published`
- `acknowledged`
- `acknowledged_with_exceptions`
- `escalated`
- `expired`

这里刻意不使用：

- `approved`
- `rejected`

因为这会把交接 receipt 错配成审批动作。

### 6.3 与 follow-up item 的关联方式

更合理的关系是：

- `handoff_receipt.follow_up_item_ids` 指向本次交接包覆盖的待办
- `follow_up_item.owner_assignment.accepted_owner` 仍单独表达 item 级接手
- 交接包可以 `acknowledged`，同时仍有若干 `follow_up_item` 尚未 accepted

这能更真实地反映制造现场的交接节奏。

## 7. 任务详情与 query 方向

### 7.1 Task detail 方向

Phase A 最适合先在 `GET /api/v1/tasks/{task_id}` 中增加：

- `handoff_receipt`
- `handoff_receipt_summary`

其中 `handoff_receipt_summary` 可以至少包含：

- `status`
- `published_at`
- `required_ack_by`
- `acknowledged_at`
- `covered_follow_up_count`
- `unaccepted_follow_up_count`
- `exception_flag`

### 7.2 为什么先放任务详情

原因很直接：

1. 交接 receipt 先天就与单次交接任务强关联。
2. 当前任务详情主链已经稳定。
3. file / SQLite repository 现阶段都适合先承接只读字段扩展。
4. 这样可以在不新增大量 endpoint 的前提下先验证交接语义。

### 7.3 跨班次 query 方向

当交接 receipt 进入 Phase B 之后，才值得考虑聚合查询资源，例如：

- `GET /api/v1/handoff-receipts`

第一版聚合查询最值得支持的过滤维度包括：

- `shift_id`
- `receipt_status`
- `receiving_role`
- `receiving_actor`
- `overdue_only`
- `has_exceptions`
- `escalated_only`

这层主要回答：

- 哪些交接包还没被接收
- 哪些交接包已接收但带异议
- 哪些交接包已超时未确认

## 8. Governance 与审计方向

### 8.1 Governance 边界

交接 receipt 最需要强调的治理边界是：

- `Production Supervisor` 负责发送内容的真实性和完整性
- `Incoming Shift Supervisor` 负责确认是否接收
- receipt 的存在不等于高风险事项已被妥善处置
- 任何涉及停线、安全、质量升级的动作仍需走正式 governed workflow

### 8.2 后续最值得补的审计事件

交接 receipt 进入实现后，至少应支持：

- `handoff_published`
- `handoff_viewed`
- `handoff_acknowledged`
- `handoff_acknowledged_with_exceptions`
- `handoff_receipt_expired`
- `handoff_receipt_escalated`

这些事件更适合支撑：

- 交接回放
- 责任解释
- receipt projection rebuild

而不是替代 follow-up item 的业务状态。

## 9. 分阶段实施建议

### Phase A. Receipt Read Model Baseline

目标：

- 让单次交接任务第一次拥有正式的回执对象

建议交付：

1. 冻结 `handoff_receipt` 最小字段集。
2. 明确 `published / acknowledged / acknowledged_with_exceptions / expired` 状态语义。
3. 在 `tasks/{task_id}` 中返回 `handoff_receipt`。
4. 保持所有对象为 read-only / draft 输出。

退出标准：

- 用户能在任务详情中看见“这份交接包是否已被接住”。

### Phase B. Explicit Acknowledgement and Audit

目标：

- 让交接 receipt 从只读对象推进到可审计的确认动作

建议交付：

1. 设计显式 acknowledgement action，而不是复用审批接口。
2. 写入 receipt 状态变化的 audit events。
3. 支持接收方记录 exceptions note。
4. 与 follow-up item 的 accepted owner 分开维护。

退出标准：

- 用户能分清“交接包已确认”和“遗留事项已接手”的差异。

### Phase C. Cross-shift Receipt Queue

目标：

- 让交接 receipt 进入跨班次 backlog 和超时监控

建议交付：

1. 增加 `handoff receipt` 聚合 projection。
2. 支持按班次、接收角色、状态和超时查询。
3. 支持 receipt overdue 与高风险 follow-up 未升级的联动提醒。
4. 让交接 receipt 与 follow-up / SLA query 形成联动视图。

退出标准：

- 用户能直接查询“哪些交接包还没人接、哪些交接包接了但有异议”。

## 10. 当前最值得立即推进的动作

如果只选 4 个最该落地的动作，建议顺序如下：

1. 冻结 `handoff_receipt` 最小字段和状态语义
2. 明确 `handoff receipt` 与 `follow_up accepted owner` 的边界
3. 明确 receipt 超时与高风险事项升级之间的治理边界
4. 定义第一版 task detail 中 `handoff_receipt_summary` 的字段集

## 11. 结论

交接场景真正需要的，不只是结构化 follow-up，还需要一层独立的 `receipt / acknowledgement` 对象来表达：

- 交接包是否已发出
- 是否已被接收班次正式接住
- 是否存在接收异议或超时未确认

只有把这层对象单独做出来，`班次交接摘要与待办提取` 才会从“有摘要、有待办”进一步走向“有接收闭环”的日常协同能力。
