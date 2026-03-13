# Alert Triage Follow-up Draft Seeding Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

只把 `follow_up_items` 做到 `shift handoff` 还不够，因为那只能证明平台能承接交接类动作草稿，不能证明它也能承接高时效事件协同里的 owner-action。

这一层单独拿出来，就是为了让 `alert triage` 第一次真正出现 follow-up draft。

## 2. 这一阶段的创新点

这一阶段的关键，是没有去做完整的 owner acceptance 或 assignment 系统，而是先让 `alert triage` 在已有 cluster draft 的同一条任务主链里，再生成一条受控 follow-up。

这意味着平台开始明确：

- follow-up seeding 可以跨两个高频场景复用同一条 task detail 主链
- 事件分诊对象和 owner-action 对象可以先共存，再逐步补 action
- 第二条真实 follow-up draft 不需要等新的写接口或新状态机

## 3. 这如何改变世界

制造现场很多异常响应失败，不是因为没有告警，而是因为系统只会显示“有告警”，不会把“下一步谁先接住”明确成结构化对象。

只要 `alert triage` 一直没有 follow-up draft，平台就更像“会聚类的系统”，而不是“能推进一线响应动作的系统”。

## 4. 对自己的要求

- 不把第二条 follow-up draft 夸大成通用 follow-up 自动生成能力完成
- 不让 alert triage follow-up 破坏既有 shift handoff 行为
- 不为了这一步引入过重的新接口或动作状态机

## 5. 已经验证的事实

- `alert triage` 请求现在能返回 1 条 seeded `follow_up_item`
- 同一请求仍会保留 1 条 seeded `alert_cluster_draft`
- sandbox-safe file mode 重启后仍能回读该 follow-up draft
- 现有 `shift handoff` 与默认高风险异常路径仍保持原行为，不受影响

## 6. 这次做对了什么

这次做对的地方，是没有急着去做 cross-task follow-up queue 或 owner acceptance action，而是先让第二条高频场景验证“task detail 主链能否同时承接 cluster 和 follow-up”。

这样后续不论是扩 acknowledgement，还是扩 follow-up owner action，都能沿同一条低风险、可验证的路线继续推进。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `follow_up_items` 第一次从“只覆盖交接”变成“同时覆盖交接和告警分诊”。

这会直接影响后续路线：

- 高频 follow-up 主线开始具备跨场景复用能力
- `alert triage` 会更像真实行动入口，而不只是异常聚类视图
- owner acceptance / assignment action 终于有了更合理的下一刀代码落点
