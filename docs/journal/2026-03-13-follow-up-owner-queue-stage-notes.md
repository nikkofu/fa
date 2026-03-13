# Follow-up Owner Queue Stage Notes - 2026-03-13

## 1. 为什么这一层必须单独拿出来

有了 task-scoped `accepted owner` 之后，平台已经能回答“这条 follow-up 被谁接了”。但现场协同真正更高频的问题不是单条 item，而是：

- 这个角色现在手里还有哪些待办
- 哪些待办更应该先处理
- 哪类待办在跨任务层面开始堆积

如果没有 cross-task queue，系统仍然只是“任务详情更丰富了”，还不是“运营视角可用的协同系统”。

## 2. 这一阶段的创新点

这一阶段的关键，是没有一上来做 dedicated projection、独立 backlog 存储或新的 item aggregate，而是先复用已经稳定存在的 `TrackedTaskState`：

- repository 增加 `list()`
- orchestrator 在读层聚合 `follow_up_item`
- API 暴露 `GET /api/v1/follow-up-items`

这意味着平台开始明确：

- cross-task owner queue 是现有 task-scoped read model 的上一层读视图
- queue read 可以先成立，不必先等更重的 projection 基础设施
- `effective_sla_status` 可以先在 query 时计算，而不是把全部结果预写死

## 3. 这如何改变世界

制造现场很多系统都能给出单任务明细，但真正让班组长和值班角色频繁打开的，是“现在我这边还有哪些没处理”的队列视图。

只要没有 cross-task queue，FA 就更像“会规划、会补详情的系统”。有了这一步，它才开始接近“可用于高频调度和接手跟踪的系统”。

## 4. 对自己的要求

- 不把 repository 扫描版 queue 误包装成最终态架构
- 不把 recommended owner 与 accepted owner 混写成同一层过滤语义
- 不为了第一版 queue 同时引入一批 blocked / risk / aging 复杂投影

## 5. 已经验证的事实

- `GET /api/v1/follow-up-items` 现在能同时返回 `shift handoff` 与 `alert triage` 的跨任务 follow-up item
- queue 默认会把更早到期的 item 排在前面
- `owner_id` 过滤会只返回已被对应 actor 接手的 item
- `source_kind` 过滤会只返回指定场景来源的 item
- sandbox-safe file mode 重启后仍可回读 queue 结果

## 6. 这次做对了什么

这次做对的地方，是没有急着为 queue 开新表、写同步器、造额外 worker，而是先让跨任务读能力在现有存储和 smoke 基线上落地。

这样后续如果继续做 overdue / blocked / escalation projection，就有了真实的读 API、真实的排序规则和真实的过滤需求作为锚点，而不是继续停留在概念上。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 FA 第一次开始回答跨任务 backlog 问题，而不只是回答单任务详情问题。

这会直接影响后续路线：

- owner queue 已经有了可用的 API 入口
- overdue / blocked / escalation 视图可以沿同一条读层链路继续推进
- 是否需要 dedicated projection，将来可以基于真实查询需求而不是猜测来决定
