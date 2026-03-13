# M1 Follow-up and SLA Model Alignment Checklist

## 1. 文档目的

本文件用于把 `follow-up owner / SLA` 从高频功能地图里的优先事项，推进到可实施的通用模型对齐层。

它回答 5 个问题：

1. 当前平台已经有哪些对象可以部分复用。
2. `follow-up / owner / due date / blocked status / SLA` 分别缺什么正式表达。
3. 这些缺口会如何影响 `班次交接`、`告警分诊` 和后续质量场景。
4. 当前 API 能先承接什么，不能承接什么。
5. 后续模型应按什么顺序进入实现。

本文件不是代码实现说明，也不是“平台已经支持 follow-up / SLA”的声明。它是后续领域模型、API、evidence、audit 和聚合视图演进的对齐清单。

## 2. 为什么这层模型必须先补

当前高频功能已经清楚暴露出一个共同问题：

- 交接场景需要遗留 follow-up
- 告警场景需要分诊后的 next step
- 质量和异常场景需要明确谁接手、何时完成、超时如何升级

如果平台没有正式的 follow-up / SLA 对象，就会出现三个后果：

1. 输出只能停留在摘要文本，不能稳定进入闭环。
2. owner 与 due date 只能散落在自然语言里，无法查询和升级。
3. 高风险任务与日常协同任务之间缺少可追踪的衔接层。

## 3. 当前平台基线快照

| 对象 | 当前状态 | 对 follow-up / SLA 的意义 |
| --- | --- | --- |
| `TaskRequest` | 已支持任务标题、描述、风险、角色、系统目标等通用字段 | 能承接“一个任务”，但不能表达多个 follow-up item |
| `TaskRecord` | 已支持生命周期状态和计划结果 | 能表示任务主状态，但不能表示 task 内部待办的 owner / due date / blocked status |
| `PlannedStep.owner` | 已支持步骤 owner | 这是执行计划 owner，不等于业务 follow-up owner |
| `WorkflowGovernance` | 已支持责任矩阵、approval strategy 和 fallback actions | 能表达责任边界，但不能表达具体待办的接手人与到期时间 |
| `TaskEvidence` | 已支持来源、摘要、payload 与时间戳 | 能记录 follow-up 来源线索，但不能表达 follow-up 本体 |
| `AuditEvent` | 已支持关键动作回放 | 能记录后续 SLA 变化，但当前没有对应业务对象可挂接 |

## 4. 通用对象对齐清单

### 4.1 Follow-up Item

`follow-up item` 应至少回答以下问题：

- 它是从哪条摘要、告警、审批或异常里提取出来的
- 它的下一步是什么
- 谁是建议 owner，谁是最终确认 owner
- 它是否只是提醒，还是必须进入正式任务
- 它当前是否被阻塞

当前状态：

- 只能存在于 `description`、`evidence.payload` 或自由文本说明中

主要差距：

- 没有一等 `follow-up item id`
- 没有来源引用与状态字段
- 无法把一个任务内的多个 follow-up 稳定建模

### 4.2 Owner Assignment

`owner` 在这里不应只表示“计划步骤由谁执行”，而应区分：

- recommended owner
- accepted owner
- current owner
- escalation owner

当前状态：

- 只有 `PlannedStep.owner`

主要差距：

- 不能表达业务待办的正式接手人
- 不能表达 owner 从推荐到确认的变化
- 不能表达“尚未接收”与“已确认接收”的状态差异

### 4.3 Due Date / Expected Window

高频协同场景里，`due date` 不一定是严格日历截止点，也可能是：

- 下一班前
- 2 小时内
- 当日关班前
- 本工单结束前

当前状态：

- 当前任务模型没有 `due_at / expected_by / target_window`

主要差距：

- 时间要求只能留在自然语言中
- 无法按时限查询或排序
- 无法形成逾期判断基础

### 4.4 Blocked Status

`blocked status` 应至少区分：

- 尚未开始
- 进行中
- 已完成
- 被阻塞
- 等待确认
- 已升级

当前状态：

- 任务级别只有主状态机

主要差距：

- 不能表达“任务整体未结束，但其中某条 follow-up 被阻塞”
- 不能表达阻塞原因、依赖对象与恢复条件

### 4.5 SLA Policy

`SLA` 不应只是一条静态截止时间，而应至少包括：

- 适用对象范围
- 起算时间
- 目标响应时限或完成时限
- 逾期后的升级规则
- 哪些场景仅提醒，哪些场景必须升级

当前状态：

- governance 文档中已有 `SLA / overdue policy` 方向性要求
- 代码与 API 中没有正式 `SLA policy` 对象

主要差距：

- 无法将 SLA 作为任务或 follow-up 的正式治理对象
- 无法稳定地进行 overdue 评估和升级建议

## 5. 当前代码层面的关键缺口

当前代码与 follow-up / SLA 最相关的关键事实是：

1. `TaskRequest` 只能描述单个任务请求，没有待办列表字段。
2. `TaskRecord` 只有任务主状态，没有子项状态。
3. `PlannedStep.owner` 是执行步骤 owner，不是 follow-up owner。
4. `TaskEvidence` 可以保存线索，但不能表达“这条线索已经变成正式待办”。
5. 审计主链已经存在，但没有 `follow-up created / assigned / blocked / overdue / resolved` 这类事件对象。

这意味着平台现在已经有闭环骨架，但还没有“把协同动作变成正式对象”的那一层。

## 6. 与当前高频功能的对齐

### 6.1 班次交接场景

`班次交接摘要与待办提取` 对 follow-up / SLA 的最直接要求是：

- 同一条交接摘要里可能包含多条遗留事项
- 每条遗留事项都需要 owner 建议
- 部分事项只需提醒，部分事项必须进入正式任务
- 下一班组接收与否需要状态表达

如果没有通用 follow-up 对象，交接场景就只能停留在“摘要里写了几条下一步”。

### 6.2 告警分诊场景

`产线告警聚合与异常分诊` 对 follow-up / SLA 的最直接要求是：

- 一组告警会产生多个 next step
- 每个 next step 可能归属于不同角色
- 高时效告警需要用 SLA 决定是否升级
- 同一告警簇的后续跟进不能丢回自由文本

如果没有通用 follow-up 对象，告警场景就只能停留在“建议怎么做”，无法稳定进入后续跟踪。

### 6.3 质量与异常场景

后续 `质量偏差`、`重复异常复盘`、`关键物料短缺` 等场景同样需要：

- owner 明确
- due date 明确
- blocked reason 明确
- overdue escalation 明确

因此这不是单场景需求，而是平台通用协同模型需求。

## 7. API 对齐清单

### 7.1 当前已经可复用的接口

| 接口 | 当前可复用部分 | 局限 |
| --- | --- | --- |
| `POST /api/v1/tasks/intake` | 可以承接高频协同任务主链 | 不能提交结构化 follow-up items |
| `GET /api/v1/tasks/{task_id}` | 可以回看任务状态与 evidence | 不能结构化返回 follow-up 列表 |
| `GET /api/v1/tasks/{task_id}/evidence` | 可以承接 follow-up 线索来源 | 不能区分线索和正式待办 |
| `GET /api/v1/tasks/{task_id}/governance` | 可以返回责任矩阵与策略边界 | 不能表达待办 owner / SLA policy |
| `GET /api/v1/tasks/{task_id}/audit-events` | 可以承接后续对象变更回放 | 当前没有 follow-up 专属事件种类 |

### 7.2 后续 API 最可能需要补的对象

后续不一定要立刻新增 endpoint，但至少需要逐步补齐以下对象：

- `follow_up_items`
- `owner_assignment`
- `due_window` 或 `due_at`
- `blocked_reason`
- `sla_policy`
- `sla_status`

### 7.3 当前阶段不必立刻新增的接口

在模型未冻结前，不需要立刻新增一批 follow-up 专属 API。

更合理的顺序是：

1. 先冻结对象模型和状态语义。
2. 再决定这些对象应进入 `tasks/{task_id}` 还是独立资源。
3. 最后再补查询、更新和升级接口。

## 8. Audit 与 Governance 对齐清单

### 8.1 未来应补的关键事件

当 follow-up / SLA 对象进入实现后，至少应支持以下审计事件：

- follow-up created
- owner recommended
- owner accepted
- due window assigned
- blocked reason updated
- SLA overdue detected
- escalation suggested
- escalation confirmed
- follow-up completed

### 8.2 与 governance 的关系

follow-up / SLA 不是独立于治理存在的。它需要清楚区分：

- recommendation
- acceptance
- escalation
- approval-required action

否则平台很容易把“提醒谁跟进”错误地写成“替业务分配责任”。

## 9. 分阶段实施建议

### Phase A. Follow-up Item Baseline

目标：

- 先让高频协同输出拥有正式待办对象，而不是纯文本下一步

建议交付：

1. 冻结 `follow-up item` 的最小字段集。
2. 区分 recommended owner 与 accepted owner。
3. 明确 `draft / accepted / blocked / completed / escalated` 等最小状态。
4. 让对象先以 read-only / draft 形式出现在任务结果中。

退出标准：

- `班次交接` 和 `告警分诊` 都能输出结构化 follow-up item 列表。

### Phase B. Due Window and SLA Policy Baseline

目标：

- 让 follow-up 不只是“谁做”，还包含“何时做”和“何时升级”

建议交付：

1. 增加 `due window / due_at` 表达。
2. 增加最小 `sla policy` 与 `sla status` 对象。
3. 区分“提醒型 SLA”和“必须升级型 SLA”。
4. 让 overdue 判断进入审计和治理输出。

退出标准：

- 平台能解释为什么某条 follow-up 已逾期、该升级给谁。

### Phase C. Cross-task Aggregation and Query

目标：

- 让 follow-up / SLA 真正成为平台级协同对象，而不是只挂在单任务下

建议交付：

1. 支持跨任务 follow-up 聚合视图。
2. 支持按 owner、status、due window、overdue 查询。
3. 支持阻塞原因和依赖关系回看。
4. 与事件驱动输入、时间线 evidence 形成联动。

退出标准：

- 用户能从“一个任务的摘要”走到“整个班次或整条产线的待办与逾期视图”。

## 10. 当前最值得立即推进的实现项

如果只选 3 个最该落地的动作，建议顺序如下：

1. 冻结 `follow-up item` 的最小字段和状态模型
2. 定义 `owner assignment` 与 `due window / SLA policy` 语义
3. 明确这些对象如何进入 `tasks/{task_id}` 的读模型

## 11. 结论

当前平台最缺的，不只是一个“提醒功能”，而是一层正式的协同对象模型。

在这层模型补齐之前：

- `班次交接` 只能给出文本化遗留事项
- `告警分诊` 只能给出文本化 next step
- 后续质量和异常场景也很难把责任、时限和升级稳定表达

因此，`follow-up owner / SLA` 不是锦上添花，而是把高频 Agentic 功能从“能总结”推进到“能协同闭环”的关键模型层。
