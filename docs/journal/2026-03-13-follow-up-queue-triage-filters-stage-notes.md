# Follow-up Queue Triage Filters Stage Notes - 2026-03-13

## 1. 为什么这一层必须单独拿出来

owner queue 解决的是“谁手里有什么”，但现场真正更高频的问题往往是：

- 哪些待办被卡住了
- 哪些已经需要升级
- 哪些高风险高优先级事项必须先看

如果没有这层 triage filters，queue 只是 backlog list，还不是现场运营真正会反复打开的值班视图。

## 2. 这一阶段的创新点

这一阶段的关键，不是扩新的对象，而是让已有 queue 开始回答更贴近值班现实的问题：

- `blocked_only`
- `escalation_required`
- `due_before`
- `risk`
- `priority`

这意味着平台开始明确：

- owner queue 和 triage queue 可以先共享同一条读层主链
- 很多高频运营问题不一定要等 dedicated projection 才能成立
- 只要正式字段已经存在，就可以先把 query 语义拉起来验证

## 3. 这如何改变世界

制造现场真正影响响应速度的，不是“看见有一堆待办”，而是“快速把最应该先处理的那几条筛出来”。

只要 queue 还不能按 blocked、escalation、risk、priority 过滤，FA 就更像一个能列清单的系统。有了这一步，它开始接近一个能支撑现场 triage 的系统。

## 4. 对自己的要求

- 不把 triage filters 误夸大成 SLA monitoring 平台
- 不为了 filter 能力倒逼写接口先行
- 不引入超出当前正式字段范围的推断逻辑

## 5. 已经验证的事实

- `GET /api/v1/follow-up-items?blocked_only=true` 可以只返回被阻塞的 follow-up item
- `GET /api/v1/follow-up-items?escalation_required=true` 可以只返回需要升级的 follow-up item
- `GET /api/v1/follow-up-items?risk=high&priority=expedited&due_before=...` 可以只返回高风险高优先级且更早到期的 item
- sandbox-safe file mode 重启后仍可回读 triage filter 结果

## 6. 这次做对了什么

这次做对的地方，是没有在 owner queue 之后马上跳到更重的 projection，而是先补最有现场价值的一层 query 维度。

这样后续如果真要做 dedicated projection 或 SLA monitoring，就已经有了真实的 filter 语义、真实的验证数据和真实的使用路径，而不是拍脑袋设计。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 FA 的 follow-up queue 第一次开始具备“值班 triage”能力，而不是只有“owner backlog”能力。

这会直接影响后续路线：

- queue 已经能服务更高频的现场筛选问题
- dedicated projection 何时引入可以基于真实 triage 需求决策
- handoff receipt overdue queue 与 SLA monitoring view 的推进边界会更清晰
