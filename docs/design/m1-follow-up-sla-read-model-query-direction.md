# M1 Follow-up and SLA Read Model and Query Direction

## 1. 文档目的

本文件把 `follow-up / SLA` 从“模型对齐清单”继续推进到“read model 和 query 方向说明”。

它重点回答 5 个问题：

1. 哪些 follow-up / SLA 对象应该先进入 `tasks/{task_id}` 的任务级读模型。
2. 哪些能力必须走跨任务聚合查询，而不能继续塞在任务详情里。
3. 当前存储、审计和 API 基线分别适合承接什么，不适合承接什么。
4. `班次交接`、`告警分诊`、`质量偏差` 等场景应如何共享同一层 read model。
5. 后续实现应如何分阶段推进，而不是一开始就扩一批写接口。

本文件不是代码实现说明，也不是“平台已经有 follow-up 查询 API”的声明。它是后续任务读模型、聚合查询、SLA 评估和审计演进的方向说明。

## 2. 方向原则

进入 read model 和 query 设计时，继续坚持以下原则：

- 先做 read-only / draft 输出，不先做大规模写操作接口。
- 先把 task-scoped read model 接进现有 `tasks/{task_id}` 主链，再做跨任务聚合查询。
- 通用 `follow-up item` 只表达待办、owner、due window、blocked、SLA，不吞掉场景专属对象。
- `班次交接` 的 `receipt / acknowledgement` 仍应保留为交接专属对象，而不是强塞进通用 follow-up 状态。
- `告警分诊` 的 `alert cluster / triage draft` 仍应保留为告警专属对象，而不是直接等同于 follow-up item。
- overdue 判断必须来自正式字段和策略对象，而不是继续靠摘要文本推断。

## 3. 当前平台基线快照

| 对象 | 当前状态 | 对 read model / query 的意义 |
| --- | --- | --- |
| `TrackedTaskState` | `GET /api/v1/tasks/{task_id}` 返回 `correlation_id / planned_task / context_reads / evidence` | 任务级读模型已有稳定挂载点，适合先承接 `follow_up_items` |
| task repository | 文件模式和 SQLite 模式都直接持久化整个 `TrackedTaskState` JSON | Phase A 往任务详情加 read-only 字段成本低，不必先改存储架构 |
| SQLite `tasks` table | 当前只索引 `task_id / correlation_id / updated_at / payload_json` | 跨任务 owner / overdue 查询不适合继续靠任务 JSON blob 扫描 |
| `AuditEventQuery` | 当前只支持 `task_id / approval_id / correlation_id / kind` 过滤 | 审计适合回放，不适合做 follow-up backlog / SLA queue 查询 |
| `TaskEvidence` | 已有 `connector / record_kind / source_ref / observed_at / payload` | 能作为 follow-up 来源线索，但没有 `evidence_id`，引用粒度仍偏弱 |
| `WorkflowGovernance` | 仍按任务级输出责任矩阵、审批策略和 fallback actions | 可为 follow-up 提供升级边界，但不是 item 级 owner / SLA 本体 |

## 4. 推荐的 read model 分层

### 4.1 Layer A: Task-scoped Follow-up View

第一层最适合先进入现有 `GET /api/v1/tasks/{task_id}`。

原因很直接：

- 当前任务详情已经是平台主读模型。
- `TrackedTaskState` 已被 repository 全量持久化。
- `班次交接`、`告警分诊`、`质量偏差` 都需要先在单任务内看到结构化 follow-up 结果。

建议先引入一个任务级 read model 容器，例如：

- `follow_up_view`
- `follow_up_items`
- `follow_up_summary`
- `sla_summary`

其中最核心的是 `follow_up_items`。

### 4.2 Layer B: Cross-task Follow-up Queue

第二层不应继续挂在单任务详情里，而应是跨任务聚合读模型。

它面向的问题是：

- 当前谁手里有哪些未完成事项
- 哪些事项已经逾期
- 哪些事项被阻塞
- 哪些事项需要升级

这类问题如果继续靠遍历 `tasks/{task_id}` 或 audit replay 来回答，查询成本和语义都会越来越差。

因此需要单独的 queue / projection 读模型。

### 4.3 Layer C: SLA Monitoring View

第三层是更偏治理和运行监控的视图。

它关注的不是单条待办内容，而是：

- 逾期判断
- 升级触发
- 风险分层
- backlog aging
- 哪些队列正在堆积

这层可以复用 follow-up item，但不应与 task detail 或 owner queue 混在一起。

## 5. Task 级 read model 方向

### 5.1 推荐的最小对象

任务级最小 `follow_up_item` 建议至少包含：

| 字段 | 作用 | 当前是否已有直接承载点 | 建议 |
| --- | --- | --- | --- |
| `id` | 稳定识别单条待办 | 无 | 需要新增 |
| `title` | 简短待办标题 | 只能放文本 | 需要新增 |
| `summary` | 给前端和 API 调用方的简要说明 | 可从文本复用 | 需要新增正式字段 |
| `source_kind` | 区分交接、告警、质量、异常等来源 | 无 | 需要新增 |
| `source_task_id` | 关联 originating task | 可复用 `task_id` | 需要新增正式字段 |
| `source_refs` | 引用 evidence / connector / alert / note 来源 | 只有 `source_ref` 字符串可用 | 先用轻量引用数组 |
| `status` | 如 `draft / accepted / in_progress / blocked / completed / escalated` | 无 | 需要新增 |
| `owner_assignment` | recommended / accepted / current / escalation owner | 无 | 需要新增嵌套对象 |
| `due_window` | 响应或完成时限 | 无 | 需要新增嵌套对象 |
| `sla_status` | `on_track / due_soon / overdue / escalation_required` | 无 | 需要新增嵌套对象 |
| `blocked_reason` | 阻塞说明与依赖对象 | 无 | 需要新增 |
| `created_at / updated_at` | 回放和排序基础 | 任务级有，但 item 级没有 | 需要新增 |

### 5.2 推荐的 summary 对象

任务级 `follow_up_summary` 不需要很复杂，但建议至少有：

- `total_items`
- `open_items`
- `blocked_items`
- `overdue_items`
- `escalated_items`
- `last_evaluated_at`

这样任务详情页和 API 调用方无需每次自己遍历所有 items 才能得到总体状态。

### 5.3 为什么应先进入 `tasks/{task_id}`

当前代码基线决定了 task 级 read model 是最自然的第一步：

1. `TrackedTaskState` 已经是统一 API 响应对象。
2. file repository 和 SQLite repository 都直接持久化整个任务状态。
3. 现有前台与 smoke 路径已经围绕任务主链组织。
4. 任务级 read model 可以先做到只读，不要求马上补写接口。

因此，Phase A 最合理的做法不是新开很多 endpoint，而是先让 `tasks/{task_id}` 能返回正式的 `follow_up_items`。

## 6. 跨任务 query 方向

### 6.1 为什么不能继续靠当前 `tasks` 表和 audit 查询

当前 SQLite `tasks` 表只保存：

- `task_id`
- `correlation_id`
- `updated_at`
- `payload_json`

当前 audit 查询只支持：

- `task_id`
- `approval_id`
- `correlation_id`
- `kind`

这意味着：

- 没法高效按 owner 查询未完成待办
- 没法高效按 due window 查询即将逾期事项
- 没法高效按 blocked / overdue / escalated 查询
- 没法高效回答“某条产线、某个角色当前 backlog 是什么”

所以跨任务 query 不应建立在“扫描任务 JSON + 回放审计”这条路上，而应建立在 dedicated projection 上。

### 6.2 推荐的第一条聚合查询资源

在 API 方向上，第一条最值得补的聚合读资源是：

- `GET /api/v1/follow-up-items`

原因是它最贴近真实运营问题，也最容易被多个场景复用。

第一版不需要支持写操作，只需支持聚合读取与筛选。

### 6.3 推荐的第一批查询维度

第一版建议支持以下 query filters：

- `task_id`
- `source_kind`
- `status`
- `owner_id`
- `owner_role`
- `overdue_only`
- `blocked_only`
- `escalation_required`
- `due_before`
- `risk`
- `priority`

这批维度已经足以支撑：

- 班次交接遗留查看
- 告警分诊后续跟踪
- 高风险待办升级前排查

### 6.4 推荐的默认排序

第一版默认排序建议优先考虑：

1. `escalation_required` 优先
2. `overdue` 优先
3. `due_at / due_window` 更近优先
4. `task priority` 更高优先
5. `updated_at` 更新更近优先

这样 query 结果更符合现场协同和值班视角，而不是数据库视角。

## 7. 场景共享与边界

### 7.1 哪些对象应共享

以下对象应被 `班次交接`、`告警分诊`、`质量偏差` 等场景共享：

- `follow_up_item`
- `owner_assignment`
- `due_window`
- `sla_policy`
- `sla_status`
- `blocked_reason`

### 7.2 哪些对象不应硬塞进通用模型

以下对象不应直接并入通用 follow-up 状态机：

- `班次交接` 的 `receipt / acknowledgement`
- `告警分诊` 的 `alert cluster / triage draft`
- 高风险 governed task 的正式 approval record

更合理的关系是：

- 这些对象保留各自场景语义
- `follow_up_item` 通过 `source_kind / source_refs / source_task_id` 与它们关联
- 通用 backlog 和 SLA 查询只消费稳定的 follow-up 结果

## 8. SLA 评估方向

### 8.1 应存什么，算什么

SLA 层不建议把所有结果都持久化成静态文本。

更合理的划分是：

- 持久化 `due_window / policy / owner / status / escalation_owner`
- 评估得出 `on_track / due_soon / overdue / escalation_required`
- 把评估结果写回 read model snapshot 与 audit

### 8.2 第一版最值得支持的 SLA 状态

第一版 `sla_status` 不必太复杂，建议先支持：

- `on_track`
- `due_soon`
- `overdue`
- `escalation_required`

这已经足够支撑高频协同场景的可见性和升级边界。

## 9. 审计方向

当前 audit 主链适合继续扩，但不能承担 backlog query 的职责。

后续最值得补的 follow-up / SLA 事件包括：

- `follow_up_created`
- `follow_up_owner_recommended`
- `follow_up_owner_accepted`
- `follow_up_due_window_assigned`
- `follow_up_blocked`
- `follow_up_unblocked`
- `follow_up_completed`
- `sla_overdue_detected`
- `sla_escalation_required`

这些事件的价值主要在于：

- 回放对象变化
- 支撑审计解释
- 为后续 projection rebuild 提供事件来源

而不是直接替代 query read model。

## 10. 分阶段实施建议

### Phase A. Task-scoped Follow-up Read Model

目标：

- 让 `tasks/{task_id}` 第一次返回正式的 `follow_up_items`

建议交付：

1. 冻结 `follow_up_item` 最小字段集。
2. 冻结 `owner_assignment / due_window / sla_status` 最小嵌套对象。
3. 在任务详情响应中增加 `follow_up_items / follow_up_summary`。
4. 保持所有对象为 read-only / draft 输出。

退出标准：

- `班次交接` 和 `告警分诊` 都能在任务详情中输出结构化 follow-up items。

### Phase B. Aggregated Queue Query

目标：

- 让 follow-up 从单任务对象推进到跨任务 backlog 视图

建议交付：

1. 建立 dedicated projection，而不是扫描任务 JSON。
2. 输出 `GET /api/v1/follow-up-items` 聚合查询。
3. 支持按 owner、status、overdue、blocked、risk、priority 查询。
4. 为高时效协同场景提供默认排序。

退出标准：

- 用户能直接查询“谁手上有什么待办、哪些已逾期”。

### Phase C. SLA Monitoring and Scenario Overlays

目标：

- 让 follow-up / SLA 真正成为平台级协同运行视图

建议交付：

1. 增加 SLA 评估与升级状态快照。
2. 补按 overdue / escalation_required 的监控视图。
3. 让 `班次交接 receipt` 和 `alert triage cluster` 与通用 follow-up 联动。
4. 形成 projection rebuild 与审计回放的一致链路。

退出标准：

- 用户能从单任务 follow-up 走到跨任务 SLA 监控，再走到具体场景对象。

## 11. 当前最值得立即推进的动作

如果只选 4 个最该落地的动作，建议顺序如下：

1. 冻结 `follow_up_item` task-scoped read model 的最小字段集
2. 明确 `accepted owner` 与 `handoff receipt` 的边界
3. 明确 `alert cluster` 与 `follow_up_item` 的边界
4. 定义第一版 `GET /api/v1/follow-up-items` 的过滤维度和默认排序

## 12. 结论

当前平台最合理的前进方向，不是立即扩一批 follow-up 写接口，而是先把 read model 分层收紧：

- 单任务内先通过 `tasks/{task_id}` 输出正式 `follow_up_items`
- 跨任务再通过 dedicated projection 提供 backlog query
- SLA 最后进入监控和升级视图

只有这样，`follow-up / SLA` 才会从“模型清单”变成平台真正可用的协同读层。
