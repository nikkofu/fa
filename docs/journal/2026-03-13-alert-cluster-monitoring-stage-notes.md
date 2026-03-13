# Alert Cluster Monitoring Stage Notes - 2026-03-13

## 1. 为什么这一层必须补上

alert cluster queue 回答的是“当前有哪些异常簇”，但产线现场真正高频的问题很快会变成：

- 现在 backlog 里有多少 open cluster
- 多少 cluster 已经进入升级候选
- 多少 cluster 仍在活动窗口，多少已经过窗还没处理

如果没有 monitoring 视图，系统仍然更像一个列表，而不是一个可用于事件运行监控的协同系统。

## 2. 这一阶段的关键做法

这一步继续坚持最窄实现：

- orchestrator 复用 repository `list()`
- monitoring 直接复用 cluster queue filter
- API 暴露 `GET /api/v1/alert-cluster-monitoring`

这让平台开始明确：

- cluster queue 回答 item list
- cluster monitoring 回答 backlog summary
- 两者是同一条读层链路，而不是两套互不相干的实现

## 3. 这如何改变世界

制造现场告警风险不只来自“有异常”，还来自“没人知道异常簇 backlog 现在积压到了什么程度”。

只要系统能快速告诉主管：

- 多少 cluster 还处于 open
- 多少已经是升级候选
- 多少还在活动窗口，多少已经 stale

它就开始从单任务分诊工具走向真正的平台级异常监控层。

## 4. 这次坚持的边界

- 不让 monitoring 引入独立写模型
- 不让 monitoring 和 queue 出现不同过滤口径
- 不把第一版 monitoring 夸大成完整 event-ingestion 平台

## 5. 已经验证的事实

- `GET /api/v1/alert-cluster-monitoring` 现在能返回最小 alert cluster monitoring 视图
- 监控结果会返回 `total / open / escalation_candidate / high_severity / active_window / stale_window / next_window_end_at`
- 监控结果会返回 `cluster_status / source_system / severity_band / triage_label / owner_role / window_state / task_risk / task_priority` 八组 bucket 统计
- `source_system=scada` 与 `open_only=true` 等过滤会直接收敛 monitoring 结果
- sandbox-safe file mode 重启后仍可回读 monitoring 结果

## 6. 这次做对了什么

这次做对的地方，是没有为了 monitoring 先造新表，而是先验证当前 cluster queue 语义能否自然支撑 summary 层。

这样后续如果真要做 dedicated projection、aging trend 或 follow-up linkage，就已经有了稳定 API 和真实测试路径，而不是停留在设计想象里。

## 7. 这一步如何继续往前推

这一步的真正意义，在于它把 FA 的 alert cluster 能力从“能列出异常簇”推进到“能看见异常簇 backlog 轮廓”。

后续路线会更清晰：

- cluster queue 和 monitoring 已经形成连续读层
- dedicated projection 何时引入，可以基于真实查询频率和数据量决定
- cluster-to-follow-up linkage、aging trend、escalation monitor 都可以沿这一层继续演进
