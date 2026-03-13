# Handoff Receipt Monitoring Stage Notes - 2026-03-13

## 1. 为什么这一层必须补上

receipt queue 解决的是“当前有哪些交接包”，但值班现场更高频的问题其实是：

- 还有多少 receipt 没被确认接收
- 哪些 receipt 已经超时
- 哪些 receipt 虽然被接收了，但带着 exceptions 或已经升级

如果没有 monitoring 视图，系统仍然更像一个交接包列表，而不是一个可用于值班监控的协同系统。

## 2. 这一阶段的关键做法

这一步继续坚持最窄实现：

- orchestrator 复用 repository `list()`
- monitoring 直接复用 receipt queue filter
- API 暴露 `GET /api/v1/handoff-receipt-monitoring`

这让平台开始明确：

- receipt queue 回答 item list
- receipt monitoring 回答 backlog summary
- 两者是同一条读层链路，而不是两套互不相干的实现

## 3. 这如何改变世界

制造现场交接风险不只来自“没人写交接”，还来自“没人知道交接 backlog 已经在什么位置积压”。

只要系统还不能快速告诉主管：

- 多少 receipt 还没被确认
- 多少已经超时
- 多少带异议
- 多少已经升级

它就还没真正进入班次协同监控层。

## 4. 这次坚持的边界

- 不让 monitoring 引入独立写模型
- 不让 monitoring 和 queue 出现不同过滤口径
- 不把第一版 monitoring 夸大成完整 aging trend 平台

## 5. 已经验证的事实

- `GET /api/v1/handoff-receipt-monitoring` 现在能返回最小 handoff receipt monitoring 视图
- 监控结果会返回 `total / open / acknowledged / unacknowledged / overdue / exception / escalated / next_ack_due_at`
- 监控结果会返回 `effective_status / receiving_role / ack_window / task_risk / task_priority` 五组 bucket 统计
- `escalated_only=true` 等过滤会直接收敛 monitoring 结果
- sandbox-safe file mode 重启后仍可回读 monitoring 结果

## 6. 这次做对了什么

这次做对的地方，是没有为了 monitoring 先造新表，而是先验证当前 receipt queue 语义能否自然支撑 summary 层。

这样后续如果真要做 dedicated projection、aging trend 或 role-load monitor，就已经有了稳定 API 和真实测试路径，而不是停留在设计想象里。

## 7. 这一步如何继续往前推

这一步的真正意义，在于它把 FA 的交接 receipt 能力从“能列出交接包”推进到“能看见交接 backlog 轮廓”。

后续路线会更清晰：

- receipt queue 和 monitoring 已经形成连续读层
- dedicated projection 何时引入，可以基于真实查询频率和数据量决定
- aging trend、role load、exception resolution monitor 都可以沿这一层继续演进
