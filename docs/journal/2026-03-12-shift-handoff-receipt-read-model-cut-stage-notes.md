# Shift Handoff Receipt Read Model Cut Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

方向文档能说明 receipt 为什么重要，但它不能自动回答实现里最麻烦的几个问题：

- 该不该新开 endpoint
- receipt 要不要等 follow-up 代码先落地
- 非交接任务该怎么兼容

这一层单独拿出来，就是为了把这些实现层分歧提前收紧。

## 2. 这一阶段的创新点

这一阶段的关键，是第一次把 `handoff_receipt` 压缩成一个能进入现有任务详情主链的最小 schema cut。

这意味着平台开始明确：

- receipt 不需要先有 write API 才能进入 task detail
- receipt 不需要等跨班次 queue 才能先成立
- receipt 不需要等非空 `follow_up_items` 才能先落 schema

## 3. 这如何改变世界

制造现场里的交接失败，常常不是因为没人会写摘要，而是因为系统没有一个稳定字段告诉你：

- 交接包到底发没发
- 对方有没有正式接住
- 目前是不是已经超时未确认

只要这层还停留在自然语言里，交接闭环就很难真正被平台承接。

## 4. 对自己的要求

- 不把 receipt schema cut 膨胀成交接子系统重构
- 不把 approval 语义偷偷复用进 receipt
- 不让 `follow_up_items` 和 receipt 互相卡住

## 5. 已经验证的事实

- 当前 `TrackedTaskState` 已是统一 task detail 载体
- 当前 `GET /api/v1/tasks/{task_id}` 可直接承接 receipt 字段扩展
- 当前 repository 通过整包 JSON 持久化，适合先做 receipt schema cut

## 6. 这次做对了什么

这次做对的地方，是承认 receipt 第一刀应该非常小：

- 两个字段
- 不加新路由
- 不加新表
- 不加 acknowledgement action

这样交接场景能先拥有正式对象，而不是继续被一堆后续能力绑住。

## 7. 这一步如何真正产生影响

这份 implementation cut note 的真正价值，在于它让 `班次交接摘要与待办提取` 第一次从“有方向”推进到了“代码入口明确、依赖关系明确”。

这会直接影响后续路线：

- `handoff_receipt` 更容易真正进入 task detail
- `follow_up_items` 和 receipt 会形成可分步推进的主线
- 交接 acknowledgement 和跨班次 queue 后续会更容易沿正确边界扩展
