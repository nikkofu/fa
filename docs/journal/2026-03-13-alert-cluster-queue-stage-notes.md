# Alert Cluster Queue Stage Notes - 2026-03-13

## 1. 为什么这层现在值得补

只把 `alert_cluster_drafts` 放在任务详情里，平台回答的仍然是“这个 triage task 有什么”。但制造现场更高频的问题其实是：

- 现在线上有哪些异常簇还在持续
- 哪些簇已经到升级边界
- 哪些簇来自 `scada`，哪些来自 `andon`

如果不能跨任务看 backlog，告警分诊仍然更像单任务辅助，而不是高频事件协同。

## 2. 这一步的关键做法

这一步继续保持最窄实现：

- orchestrator 直接复用 repository `list()`
- queue item 直接从 `TrackedTaskState.alert_cluster_drafts` 扫描生成
- API 暴露 `GET /api/v1/alert-clusters`

这让平台开始明确：

- `alert_cluster_draft` 是 task-scoped detail 对象
- `alert-clusters` 是 cross-task backlog 读层
- 二者是同一份数据的两种视角，而不是两套不同模型

## 3. 这次把什么变成了更高频能力

制造现场的告警问题，很多时候并不是“单条告警不够详细”，而是“没人能快速看见全厂当前有哪些异常簇在堆积”。

只要平台能稳定回答：

- 当前 open cluster 有哪些
- 哪些 cluster 是升级候选
- 哪些 cluster 来自哪条线、哪个系统、哪个时间窗

它就开始从 task-centric orchestration 走向真正的 event backlog coordination。

## 4. 这次坚持的边界

- 不为第一版 queue 引入 dedicated projection
- 不把 queue 扩大成 monitoring 或 ingestion 平台
- 不让 `window_from / window_to` 变成模糊不清的过滤口径

## 5. 已经验证的事实

- `GET /api/v1/alert-clusters` 现在能跨任务返回 alert cluster queue
- queue 默认会把升级候选且高 severity 的 cluster 排在更前面
- `source_system / line_id / triage_label / escalation_candidate / window_from / window_to / open_only` 过滤都已可用
- sandbox-safe file mode 重启后仍可回读 queue 与过滤结果

## 6. 这次做对了什么

这次做对的地方，是没有为了做 queue 就急着引入新的 cluster projection 表，而是先验证：

- 当前 task detail 里的 draft 字段是否已经足够稳定
- repository-scan 是否已经能支撑第一版 cross-task backlog
- 哪些过滤维度是真正高频而且有解释价值的

这样后续如果真要做 dedicated projection、monitoring 或 ingestion linkage，API 形状和测试路径都已经先稳定下来了。

## 7. 这一步如何继续往前推

现在路线已经更清楚：

- task detail draft 已成立
- cross-task queue 已成立
- 下一步值得评估的是 queue 与 follow-up / monitoring 的联动，以及何时需要独立 projection

这意味着 `alert triage` 已经不只是“看到单个 task 的证据”，而是在形成平台级异常 backlog 的雏形。
