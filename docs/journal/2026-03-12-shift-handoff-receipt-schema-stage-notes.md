# Shift Handoff Receipt Task Detail Schema Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

有 implementation cut 文档，不代表交接系统已经真正有了 receipt 入口。很多项目会停在“对象设计已经想清楚”，但 API 里仍然没有正式字段，结果“是否已接住交接包”还是只能靠自然语言描述。

这一步单独拿出来，就是为了把 receipt 真正接到系统主链上。

## 2. 这一阶段的创新点

这一阶段的关键，是把 `handoff_receipt / handoff_receipt_summary` 以最小、兼容、可回读的方式接进了 `TrackedTaskState`。

这意味着平台开始明确：

- receipt task detail 不需要新 endpoint
- receipt task detail 不需要新表
- receipt task detail 可以先以空 schema 稳定成立

## 3. 这如何改变世界

制造交接里的很多损耗，并不是因为没人写摘要，而是系统没有正式位置表达：

- 这次交接有没有发布
- 是否已经有 receipt
- 这些信息能不能和任务详情一起稳定回读

只要这层没有进入正式 task detail，交接协同就很难从“说明发生过”走到“系统知道是否接住”。

## 4. 对自己的要求

- 不把空 schema 夸大成交接闭环已完整实现
- 不让兼容旧 JSON 成为事后才补的风险
- 不把 schema cut 扩大成 acknowledgement 系统重构

## 5. 已经验证的事实

- 当前 `tasks/intake` 与 `tasks/{task_id}` 都能返回 `handoff_receipt / handoff_receipt_summary`
- 当前 file / SQLite repository 都能继续整包持久化任务状态
- 旧任务 JSON 缺少新字段时仍能成功回读

## 6. 这次做对了什么

这次做对的地方，是第一刀依旧非常克制：

- 只扩 `TrackedTaskState`
- 只加默认空字段
- 只补兼容和 round-trip 测试
- 不动生命周期和路由结构

这样 receipt 主链先成立，后续确认动作和队列能力才有稳定落点。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `handoff_receipt` 第一次从文档和设计说明进入了实际 API contract。

这会直接影响后续路线：

- `alert_cluster_drafts` 可以沿同一模式继续落地
- 非空 receipt draft 填充会更容易收敛
- 高频协同对象开始真正共享同一条任务详情主线
