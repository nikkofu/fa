# Alert Cluster Linkage Triage Filters Stage Notes - 2026-03-13

## 1. 为什么这一层必须补上

如果 queue 里已经能看到 cluster 和 linked follow-up 摘要，但主管仍然没法直接问：

- 哪些 cluster 已经有人接
- 哪些 cluster 还有 follow-up 没被 accepted
- 哪些 cluster 的 linked follow-up 已经进入升级态

那它仍然更像一个数据列表，而不是一个可操作的异常分诊面板。

## 2. 这一阶段的关键做法

这一步继续坚持最窄实现：

- 不新增 route
- 不新增 projection
- 只把 linked follow-up triage filter 接到已有 queue / monitoring query 上

本阶段补的过滤只有三类：

- `follow_up_owner_id`
- `unaccepted_follow_up_only`
- `follow_up_escalation_required`

这说明平台开始明确：

- `linked_follow_up` 摘要回答“单条 cluster 的处置状态”
- triage filter 回答“整个 backlog 里先看哪一类 cluster”
- monitoring 继续复用 queue 口径，而不是另起一套

## 3. 这如何改变世界

制造现场的响应速度，不只取决于系统能不能识别异常，还取决于系统能不能把“还没人接”与“已经开始升级”的异常簇立即筛出来。

只要主管能直接过滤：

- 仍未被 accepted 的 cluster
- 已被某个 owner 接走的 cluster
- linked follow-up 已进入 escalation 的 cluster

系统就从“能看见异常”进一步走向“能压缩异常分诊时间”。

## 4. 这次坚持的边界

- 不发明新的 cluster 状态字段
- 不让 queue 和 monitoring 出现两套 follow-up filter 语义
- 不为 triage filter 提前引入 dedicated projection

## 5. 已经验证的事实

- `GET /api/v1/alert-clusters` 现在支持 linked follow-up triage filters
- `GET /api/v1/alert-cluster-monitoring` 会复用同一组 triage filters
- accepted owner、未接单状态和 escalation-required 状态都能稳定过滤 backlog
- sandbox-safe file mode 重启后仍可回读这些过滤结果

## 6. 这次做对了什么

这次做对的地方，是没有继续堆字段，而是把已有 `linked_follow_up` 摘要转成真正可操作的过滤能力。

这样做的价值很直接：

- query contract 仍然很小
- 运营用户真正高频的问题能被直接回答
- 后续如果要做 monitoring bucket 或 dedicated projection，就已经有了验证过的 triage 语义

## 7. 这一步如何继续往前推

这一步的意义在于，FA 现在不只会显示 linked follow-up，还会按 linked follow-up 的运营状态筛 alert cluster backlog。

后续路线会更清晰：

- 把 triage filter 进一步推进到 monitoring 聚合字段
- 评估 dedicated projection 是否值得引入
- 让 cluster、follow-up、SLA monitoring 形成更完整的异常运营工作台
