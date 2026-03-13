# Follow-up Task Read Model Schema Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

有 implementation cut 文档，不代表平台已经真正有了 follow-up 入口。很多项目会停在“字段设计已经想清楚”，但 API 里仍然没有正式结构，结果所有协同语义还是继续躲在 evidence 文本里。

这一步单独拿出来，就是为了把 follow-up 真正接到系统主链上。

## 2. 这一阶段的创新点

这一阶段的关键，是把 `follow_up_items / follow_up_summary` 以最小、兼容、可回读的方式接进了 `TrackedTaskState`。

这意味着平台开始明确：

- follow-up task detail 不需要新 endpoint
- follow-up task detail 不需要新表
- follow-up task detail 可以先以空 schema 稳定成立

## 3. 这如何改变世界

制造协同系统里最常见的问题之一，不是没有待办，而是系统没有正式位置表达：

- 当前有哪些 follow-up
- 当前汇总状态是什么
- 这些信息是否和任务详情一起稳定回读

只要这层没有进入正式 task detail，平台就很难从“会规划”走到“能协同”。

## 4. 对自己的要求

- 不把空 schema 夸大成 follow-up 业务逻辑完成
- 不让兼容旧 JSON 成为事后才补的风险
- 不把 schema cut 扩大成新的 backlog 系统

## 5. 已经验证的事实

- 当前 `tasks/intake` 与 `tasks/{task_id}` 都能返回 `follow_up_items / follow_up_summary`
- 当前 file / SQLite repository 都能继续整包持久化任务状态
- 旧任务 JSON 缺少新字段时仍能成功回读

## 6. 这次做对了什么

这次做对的地方，是第一刀非常克制：

- 只扩 `TrackedTaskState`
- 只加默认空字段
- 只补兼容和 round-trip 测试
- 不动生命周期和路由结构

这样 follow-up 主链先成立，后续复杂逻辑才有稳定落点。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `follow_up_items` 第一次从文档和设计说明进入了实际 API contract。

这会直接影响后续路线：

- `handoff_receipt` 和 `alert_cluster_drafts` 可以沿同一模式继续落地
- 非空 follow-up draft 填充会更容易收敛
- 高频协同对象开始真正共享同一条任务详情主线
