# Follow-up SLA Monitoring Stage Notes - 2026-03-13

## 1. 为什么这一层必须单独成立

owner queue 解决的是“现在有哪些待办”，但现场运营更高频的问题其实是：

- backlog 总量有没有开始堆积
- 哪些事项已经进入升级边界
- 哪些角色还没有接手关键待办

如果没有 monitoring 视图，系统仍然更像一个可查询队列，而不是一个能支撑日常运行监控的协同系统。

## 2. 这一阶段的关键做法

这一步没有新建 projection 表，也没有新写一套过滤语义，而是直接建立在已有 queue 之上：

- orchestrator 继续复用 repository `list()`
- monitoring 直接复用 follow-up queue filter
- API 暴露 `GET /api/v1/follow-up-monitoring`

这让平台开始明确：

- queue 和 monitor 是同一条读链路上的两个层级
- owner queue 回答 item list
- monitoring 回答 backlog summary

## 3. 这如何改变世界

制造现场的问题往往不是“没有待办”，而是“没人知道 backlog 已经在什么位置失控了”。

只要系统还不能快速告诉值班主管：

- 现在有多少 open items
- 多少已经被接手
- 多少被阻塞
- 多少已经需要升级

它就还没真正进入日常运营监控层。

## 4. 这次坚持的边界

- 不让 monitoring 引入独立的写模型
- 不让 monitoring 和 queue 出现不同的过滤口径
- 不把第一版聚合视图夸大成完整 backlog aging 系统

## 5. 已经验证的事实

- `GET /api/v1/follow-up-monitoring` 现在能返回最小 follow-up SLA monitoring 视图
- 监控结果会返回 `total / open / accepted / unaccepted / blocked / overdue / escalation_required / next_due_at`
- 监控结果会返回 `source_kind / owner_role / effective_sla_status / task_risk / task_priority` 五组 bucket 统计
- `source_kind=alert_triage` 等过滤会直接收敛 monitoring 结果
- sandbox-safe file mode 重启后仍可回读 monitoring 结果

## 6. 这次做对了什么

这次做对的地方，是没有为了 monitoring 先造一个新表，而是先验证当前 queue 语义能否自然支撑 summary 层。

这样后续如果真要做 dedicated projection、aging slices 或 SLA trend，我们已经有了稳定 API 和真实测试路径，而不是只靠设计想象。

## 7. 这一步如何继续往前推

这一步的真正意义，在于它把 FA 的 follow-up 能力从“能列出待办”推进到“能看见 backlog 轮廓”。

后续路线会更清晰：

- follow-up queue 和 monitoring 已经形成连续读层
- dedicated projection 何时引入，可以基于真实查询频率决定
- backlog aging、role load、trend snapshot 都可以沿这一层继续演进
