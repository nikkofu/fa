# Alert Cluster Draft Seeding Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

空 schema 能证明 contract 稳定了，但不能证明平台真的会生成事件分诊对象。很多系统做到这一步就会停住，最后 `alert_cluster_drafts` 永远只是空数组。

这一层单独拿出来，就是为了让 cluster draft 第一次真正非空。

## 2. 这一阶段的创新点

这一阶段的关键，是没有去做完整事件入口或通用聚类系统，而是先为 `alert triage` 这条高频、强时效场景做一条受控 cluster draft。

这意味着平台开始明确：

- alert cluster seeding 可以先从单场景起步
- task detail 主链已经足以承接真实事件分诊对象
- 非空 cluster draft 不需要等 ingestion pipeline 或独立 query API 才能出现

## 3. 这如何改变世界

制造现场的异常响应问题，很多时候不是没有告警，而是系统不能把重复、相近、需要同一角色分诊的信号归成一个“可接住的对象”。

只要 `alert_cluster_drafts` 一直为空，平台就很难从“看到了很多信号”走到“系统知道这是一簇需要分诊的异常”。

## 4. 对自己的要求

- 不把单场景 seeding 夸大成通用 alert clustering 能力完成
- 不让 cluster draft 生成破坏现有异常审批主链
- 不为了第一条 cluster draft 引入过重的新基础设施

## 5. 已经验证的事实

- `alert triage` 请求现在能返回 1 条 seeded `alert_cluster_draft`
- `alert_triage_summary` 会同步返回 cluster 总数、打开数和 escalation candidate 数量
- sandbox-safe file mode 重启后仍能回读该 seeded cluster draft
- 现有 `shift handoff` 与默认高风险异常路径仍保持原行为，不受影响

## 6. 这次做对了什么

这次做对的地方，是没有直接跳去做 raw event ingestion、独立 cluster query 或通用聚类策略，而是先让 cluster draft 在同一条高频场景里稳定进入 task detail 和持久化主链。

这样后续不论是扩更多 alert pattern，还是扩 follow-up / receipt acknowledgement，都可以继续沿“单场景受控验证”这条路线走。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `alert_cluster_drafts` 第一次从“空默认 schema”进入了“真实 draft 对象”。

这会直接影响后续路线：

- 第二类告警模式可以开始复用同一条 task detail 主链
- 现场 triage 会更像真实协同入口，而不是证据列表的旁注
- 高频事件协同对象的主链开始同时覆盖 follow-up、receipt 和 alert cluster
