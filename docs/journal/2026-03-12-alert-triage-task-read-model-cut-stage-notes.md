# Alert Triage Task Read Model Cut Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

方向文档能说明 cluster draft 为什么必要，但它不能自动回答实现里最麻烦的几个问题：

- raw alert 要不要直接挂进 task detail
- ingestion 没做之前能不能先落 schema
- cluster draft 和 follow-up 到底先做哪一个

这一层单独拿出来，就是为了把这些实现层分歧提前收紧。

## 2. 这一阶段的创新点

这一阶段的关键，是第一次把 `alert_cluster_drafts` 压缩成一个能进入现有任务详情主链的最小 schema cut。

这意味着平台开始明确：

- cluster draft 不需要先有 ingestion API 才能进入 task detail
- cluster draft 不需要先有聚合 query 才能先成立
- cluster draft 不需要把 raw alert 原样暴露给调用方

## 3. 这如何改变世界

制造现场里的告警噪音，常常不是因为没有信号，而是因为系统没有一个稳定字段告诉你：

- 哪些信号已经被归并成同一簇
- 这个簇现在有没有升级风险
- 该优先找谁处理

只要这层还停留在 evidence 文本里，告警协同就很难真正被平台接住。

## 4. 对自己的要求

- 不把 cluster schema cut 膨胀成事件系统重构
- 不把 raw alert 和 cluster draft 混成一个对象
- 不让 ingestion 缺口拖住 task detail 进展

## 5. 已经验证的事实

- 当前 `TrackedTaskState` 已是统一 task detail 载体
- 当前 `GET /api/v1/tasks/{task_id}` 可直接承接 cluster draft 字段扩展
- 当前 repository 通过整包 JSON 持久化，适合先做 cluster schema cut

## 6. 这次做对了什么

这次做对的地方，是承认 cluster draft 第一刀也应该非常小：

- 两个字段
- 不加新路由
- 不加新表
- 不加 ingestion 入口

这样告警场景能先拥有正式对象，而不是继续被后续事件系统能力绑住。

## 7. 这一步如何真正产生影响

这份 implementation cut note 的真正价值，在于它让 `产线告警聚合与异常分诊` 第一次从“有方向”推进到了“代码入口明确、边界明确、依赖关系明确”。

这会直接影响后续路线：

- `alert_cluster_drafts` 更容易真正进入 task detail
- 高频告警协同会更快从 evidence 文本进入结构化视图
- ingestion 与 cluster query 后续会更容易沿正确边界扩展
