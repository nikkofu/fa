# Follow-up Task Read Model Cut Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

很多项目做到 read model direction note 这一步就会误以为“接下来写代码就行”。但没有 implementation cut，团队真正动手时还是会陷入同一个问题：

- 字段放哪
- 类型放哪
- repository 要不要改
- smoke 要不要新开链路

这一步单独拿出来，就是为了把这些问题一次性收紧。

## 2. 这一阶段的创新点

这一阶段的关键，不是再定义 follow-up 术语，而是第一次把 `follow_up_items` 压缩成一个能进入现有主链的最小 schema cut。

这意味着平台开始明确：

- 第一刀就放在 `TrackedTaskState`
- `tasks/{task_id}` 可以零新增 endpoint 承接这次变化
- file / SQLite repository 可以继续复用整包 JSON 持久化

## 3. 这如何改变世界

制造现场里的协同系统常见问题，不是没人知道要跟进，而是系统没有一个稳定入口能把“任务 + 证据 + follow-up”一起返回。

只要 `follow_up_items` 还没进入任务详情，FA 就更像一个“会规划的系统”，而不是“能支撑后续协同”的系统。

## 4. 对自己的要求

- 不把 task detail schema cut 夸大成新的 backlog 平台
- 不让 `PlannedStep.owner` 偷偷变成 follow-up owner
- 不把交接 receipt、告警 cluster、质量 connector 问题混写在一起

## 5. 已经验证的事实

- 当前 `TrackedTaskState` 只包含 `correlation_id / planned_task / context_reads / evidence`
- 当前 `GET /api/v1/tasks/{task_id}` 已直接返回 `TrackedTaskState`
- 当前 repository 在 file / SQLite 模式下都直接持久化整个任务状态 JSON

## 6. 这次做对了什么

这次做对的地方，是没有急着新增 `follow-up` endpoint，而是先抓住任务详情这个现成主链。

这样后续第一刀代码可以非常聚焦：

- 扩 schema
- 保兼容
- 补 round-trip
- 补 smoke 断言

而不是一次性拉出一整套新接口和新 projection。

## 7. 这一步如何真正产生影响

这份 implementation cut note 的真正价值，在于它把 `follow-up / SLA` 从“读层方向正确”推进到了“代码入口明确、改动范围可控”。

这会直接影响后续路线：

- `follow_up_items` 更容易真正进入 API 主链
- `handoff_receipt` 和 `alert_cluster_drafts` 会沿同一模式继续收敛
- 高频协同功能会更快从设计文档进入可验证实现
