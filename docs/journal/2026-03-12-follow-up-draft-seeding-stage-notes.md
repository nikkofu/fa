# Follow-up Draft Seeding Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

空 schema 能证明 contract 稳定了，但不能证明平台真的会生成协同对象。很多系统做到这一步就会停住，最后 `follow_up_items` 永远只是一个空数组。

这一层单独拿出来，就是为了让 follow-up 第一次真正非空。

## 2. 这一阶段的创新点

这一阶段的关键，是没有去做一个“通用智能待办引擎”，而是先为 `shift handoff` 这条高频、低风险场景做一条受控 draft。

这意味着平台开始明确：

- draft seeding 可以先从单场景起步
- task detail 主链已经足以承接真实 follow-up
- 非空 draft 不需要等写接口或跨任务 queue 才能出现

## 3. 这如何改变世界

制造现场里很多系统的问题不是没有任务详情，而是任务详情里根本没有真正的后续动作对象。

只要 `follow_up_items` 一直为空，系统就很难告诉你：

- 现在应该谁接手
- 下一步应该在什么时间前处理
- 这条高频协同到底有没有被结构化接住

## 4. 对自己的要求

- 不把单场景 seeding 夸大成通用能力完成
- 不让高频 draft 生成破坏现有异常主链
- 不为了第一条 draft 引入过重的新基础设施

## 5. 已经验证的事实

- `shift handoff` 请求现在能返回 1 条 seeded `follow_up_item`
- `follow_up_summary` 会同步返回 `total_items / open_items`
- 现有高风险异常路径仍保持空默认 follow-up，不受影响

## 6. 这次做对了什么

这次做对的地方，是没有直接跳去做所有场景的自动 draft，而是选了一条最容易验证价值的高频协同路径。

这样后续不论是扩展 `handoff_receipt`，还是扩展 `alert_cluster_drafts`，都可以继续沿“单场景受控验证”这条路线走。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `follow_up_items` 第一次从“空默认 schema”进入了“真实 draft 对象”。

这会直接影响后续路线：

- `handoff_receipt` 和 `alert_cluster_drafts` 可以开始追求同样的非空验证
- task detail 会更像真实协同入口，而不是只读快照
- 高频场景的日常协同价值开始从文档进入可执行系统行为
