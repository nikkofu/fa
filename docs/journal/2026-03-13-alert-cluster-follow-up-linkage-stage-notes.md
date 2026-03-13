# Alert Cluster Follow-up Linkage Stage Notes - 2026-03-13

## 1. 为什么这一层值得优先补

现场主管看到 cluster backlog 后，下一句几乎总是：

- 这个 cluster 有没有 follow-up
- 有没有人接单
- 如果还没人接，或者 SLA 已经开始冒烟，现在该先盯哪个

如果 queue 只能列出 cluster 本身，系统仍然更像一个异常目录，而不是一线可执行的处置协同面板。

## 2. 这一阶段的关键做法

这一步继续坚持最窄实现：

- 不新增 endpoint
- 不新增 projection 表
- 直接在 `GET /api/v1/alert-clusters` queue item 上挂 `linked_follow_up` 摘要

linkage 规则也保持克制：

- 优先支持显式 `source_kind=alert_cluster` 且 `source_refs` 指向 `cluster_id`
- 对当前单 cluster `alert triage` task 提供兼容回退

这说明平台开始明确：

- cluster 回答事件聚合对象
- follow-up 回答后续执行对象
- queue 需要能同时把两者读出来，但不把它们混成一个对象

## 3. 这如何改变世界

制造业里的高频异常并不只是“识别信号”，而是“信号形成簇之后，是否有人真正接住了第一步处置动作”。

只要系统能在 cluster queue 上直接告诉主管：

- 这个 cluster 是否已经挂上 follow-up
- owner 是否已经 accepted
- 当前最高优先级 SLA 状态是什么

它就开始从“告警归并工具”走向“异常处置协同入口”。

## 4. 这次坚持的边界

- 不把 cluster 和 follow-up 合成一个统一写模型
- 不因为 linkage 先引入新的 projection 存储
- 不把兼容回退当成长期 contract，显式 `cluster_id` 引用仍是正确方向

## 5. 已经验证的事实

- `GET /api/v1/alert-clusters` 现在会返回最小 `linked_follow_up` 摘要
- 摘要会返回 `total / open / accepted / unaccepted / accepted_owner_ids / worst_effective_sla_status`
- follow-up owner acceptance 后，cluster queue 会同步反映 accepted owner
- 显式 `alert_cluster + cluster_id` 链路已经可用
- sandbox-safe file mode 重启后仍可回读 linkage 结果

## 6. 这次做对了什么

这次做对的地方，是把联动放在读层先验证，而不是提前发明 cluster-follow-up 新状态机。

这样一来：

- 当前 task detail 和 follow-up action 完全不用重写
- queue 能先满足现场“要看见处置状态”的高频诉求
- 后续如果真要做 projection、aging trend 或 cluster SLA 大盘，也已经有清晰的字段和测试基线

## 7. 这一步如何继续往前推

这一步的真正意义，在于 FA 现在不只会说“哪里有异常簇”，也开始说“这些异常簇后续有没有被接住”。

后续路线会更清晰：

- 把 linked follow-up 摘要推进到 monitoring 聚合口径
- 评估 dedicated projection 是否值得引入
- 让 cluster、follow-up、SLA query 真正形成一条可观测的异常处置链路
