# Alert Cluster Monitoring Linked Follow-up Aggregates Stage Notes - 2026-03-13

## 1. 为什么这一层必须补上

有了 queue linkage 和 triage filters 之后，主管仍然会继续追问：

- 现在 backlog 里到底多少 cluster 已挂 follow-up
- 多少 cluster 已经有人接
- 多少 cluster 还存在未接单的 follow-up
- 多少 cluster 的 linked follow-up 已经进入升级态

如果 monitoring 还不能直接回答这些问题，系统就依然更像一个筛选列表，而不是一个真正的异常运营监控层。

## 2. 这一阶段的关键做法

这一步继续坚持最窄实现：

- 不新增 endpoint
- 不新增 projection
- 只在现有 `GET /api/v1/alert-cluster-monitoring` 里增加 linked follow-up aggregate 字段

本阶段补的监控信息分两类：

- 五个 top-level count：`linked / unlinked / accepted / unaccepted / escalation`
- 两组 bucket：`follow_up_coverage_counts / follow_up_sla_status_counts`

这说明平台开始明确：

- queue 回答单条 cluster 的 linkage 状态
- triage filter 回答先筛哪一类 cluster
- monitoring aggregate 回答 backlog 的整体处置覆盖情况

## 3. 这如何改变世界

制造现场真正怕的不是“系统不知道有异常”，而是“系统知道有异常，但管理层看不见这些异常后续到底有没有被接住”。

只要 monitoring 能直接告诉主管：

- 多少 cluster 根本还没挂 follow-up
- 多少已经有人接
- 多少还存在未接单 backlog
- 多少已经升级

平台就从“异常聚合与筛选”进一步走向“异常处置运行监控”。

## 4. 这次坚持的边界

- 不把 monitoring aggregate 膨胀成完整 dashboard
- 不为 aggregate 提前引入 dedicated projection
- 不混淆 cluster 监控维度和 follow-up 监控维度，两者仍然通过 linked summary 接起来

## 5. 已经验证的事实

- `GET /api/v1/alert-cluster-monitoring` 现在会返回 linked follow-up aggregate 字段
- 新字段能稳定回答 linked/unlinked/accepted/unaccepted/escalation 五种 backlog 状态
- `follow_up_coverage_counts` 和 `follow_up_sla_status_counts` 会随现有过滤同步收敛
- sandbox-safe file mode 重启后仍可回读这些 aggregate

## 6. 这次做对了什么

这次做对的地方，是没有再加一个 route，而是先验证现有 monitoring contract 是否足够承载 linked follow-up 的运营层汇总。

这样做的结果很直接：

- 现有 query 口径没有分叉
- 一线和主管都能从同一个 monitoring endpoint 读到更多运营信号
- 后续如果真要做 accepted owner load bucket、projection 或 dashboard，也已经有稳定字段和测试基线

## 7. 这一步如何继续往前推

这一步的意义在于，FA 现在已经不仅能筛 cluster backlog，也开始能从 monitoring 视角看见这些 backlog 后续有没有被处置链条接住。

后续路线会更清晰：

- 增加 accepted owner / owner-load 聚合维度
- 评估 dedicated projection 是否值得引入
- 让 cluster、follow-up、SLA monitoring 逐步形成完整的异常运营工作台
