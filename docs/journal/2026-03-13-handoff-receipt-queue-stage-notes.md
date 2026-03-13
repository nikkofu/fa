# Handoff Receipt Queue Stage Notes - 2026-03-13

## 1. 为什么这一层必须单独拿出来

task-scoped `handoff_receipt` 解决的是“这一次交接包现在是什么状态”，但现场值班真正更高频的问题是：

- 哪些交接包还没人确认
- 哪些交接包已经超时
- 哪些交接包虽然被接了，但带着 exceptions

如果没有 cross-shift queue，receipt 仍然只是任务详情里的一个字段，还不是班次协同会频繁打开的运行视图。

## 2. 这一阶段的创新点

这一阶段的关键，是没有急着上 dedicated projection，而是先复用已经稳定存在的 task state：

- repository `list()`
- orchestrator 聚合 `handoff_receipt`
- API 暴露 `GET /api/v1/handoff-receipts`

这意味着平台开始明确：

- receipt queue 是 task-scoped receipt 的上一层读视图
- `expired` 可以先作为 query-time 的 effective status 存在
- cross-shift queue 可以先成立，不必先等一整套新的存储基础设施

## 3. 这如何改变世界

制造现场交接失败，很多时候不是没人写交接摘要，而是没人能快速看出“哪些交接包还没被下一班接住”。

只要没有 cross-shift receipt queue，FA 就更像“会生成交接详情的系统”。有了这一步，它开始接近一个能支撑交接班监控和追踪的系统。

## 4. 对自己的要求

- 不把 query-time `expired` 夸大成正式写入状态机
- 不把 receipt queue 和 follow-up owner queue 混写成同一个对象
- 不为了第一版 queue 同时引入 projection、monitor 和额外 worker

## 5. 已经验证的事实

- `GET /api/v1/handoff-receipts` 现在能跨任务返回 `shift handoff` 的 receipt queue
- queue 默认会把超时未确认的 receipt 排在更前面
- `overdue_only / has_exceptions / escalated_only / shift_id / receiving_actor_id` 过滤成立
- sandbox-safe file mode 重启后仍可回读 receipt queue 结果

## 6. 这次做对了什么

这次做对的地方，是没有为了 queue 能力立刻造出新的 receipt projection 表，而是先把跨班次读能力用现有 repository 和 smoke 基线跑通。

这样后续如果真要做 receipt aging monitor 或 dedicated projection，就已经有了真实 API、真实排序和真实过滤语义可以依赖。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 FA 的交接回执第一次开始回答跨班次 backlog 问题，而不只是单任务状态问题。

这会直接影响后续路线：

- 交接班值班视图已经有了可用的 API 入口
- receipt aging / exception monitoring 可以沿同一条读层链路继续推进
- dedicated projection 何时引入，可以基于真实查询需求而不是猜测来决定
