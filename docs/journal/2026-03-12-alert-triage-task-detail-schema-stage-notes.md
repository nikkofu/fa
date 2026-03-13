# Alert Triage Task Detail Schema Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

有 implementation cut 文档，不代表告警系统已经真正有了 cluster 入口。很多项目会停在“对象设计已经想清楚”，但 API 里仍然没有正式字段，结果“哪些信号已经形成异常簇”还是只能靠 evidence 文本和人工理解。

这一步单独拿出来，就是为了把 cluster draft 真正接到系统主链上。

## 2. 这一阶段的创新点

这一阶段的关键，是把 `alert_cluster_drafts / alert_triage_summary` 以最小、兼容、可回读的方式接进了 `TrackedTaskState`。

这意味着平台开始明确：

- cluster task detail 不需要新 endpoint
- cluster task detail 不需要新表
- cluster task detail 可以先以空 schema 稳定成立

## 3. 这如何改变世界

制造现场里的告警损耗，并不是因为没有信号，而是系统没有正式位置表达：

- 哪些信号已经被归并
- 当前 cluster 汇总是什么
- 这些信息能不能和任务详情一起稳定回读

只要这层没有进入正式 task detail，告警协同就很难从“收到很多信号”走到“系统知道哪一簇值得分诊”。

## 4. 对自己的要求

- 不把空 schema 夸大成告警聚类和 ingestion 已完整实现
- 不让兼容旧 JSON 成为事后才补的风险
- 不把 schema cut 扩大成事件系统重构

## 5. 已经验证的事实

- 当前 `tasks/intake` 与 `tasks/{task_id}` 都能返回 `alert_cluster_drafts / alert_triage_summary`
- 当前 file / SQLite repository 都能继续整包持久化任务状态
- 旧任务 JSON 缺少新字段时仍能成功回读

## 6. 这次做对了什么

这次做对的地方，是第一刀依旧非常克制：

- 只扩 `TrackedTaskState`
- 只加默认空字段
- 只补兼容和 round-trip 测试
- 不动 ingestion、route 和生命周期结构

这样 cluster 主链先成立，后续事件入口和聚合查询才有稳定落点。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `alert_cluster_drafts` 第一次从文档和设计说明进入了实际 API contract。

这会直接影响后续路线：

- 高频 task-detail trio 已经全部进入代码主链
- 非空 cluster draft 填充会更容易收敛
- ingestion 与 cluster query 后续会更容易沿正确边界扩展
