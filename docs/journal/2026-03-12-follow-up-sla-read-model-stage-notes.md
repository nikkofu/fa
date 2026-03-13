# Follow-up and SLA Read Model Stage Notes - 2026-03-12

## 1. 为什么这一步必须单独拿出来

很多团队在定义通用对象之后，会马上跳去写接口或写场景 demo。但如果不先想清楚这些对象应该怎样进入任务详情、怎样被跨任务查询，后续实现很容易重新退回“把内容塞进 JSON 文本里”的老路。

所以这一步真正要解决的，不是“再定义一个术语”，而是“把对象变成稳定读层”。

## 2. 这一阶段的创新点

这一阶段的关键，是把 follow-up / SLA 问题从“模型清单”推进到“读模型分层”。

这意味着平台开始明确：

- 哪些信息先跟任务详情走
- 哪些信息必须单独做聚合 projection
- 哪些语义属于通用对象，哪些仍应留给交接和告警这类场景对象

## 3. 这如何改变世界

制造业里的大量协同损耗，并不是因为大家不知道有待办，而是因为系统层根本没有一套稳定视图告诉你：

- 现在有哪些待办
- 谁真正接了
- 哪些已经逾期
- 哪些需要升级

只要没有这层读模型，平台就很难从“会总结”走到“能运营”。

## 4. 对自己的要求

- 不把 task detail 和 backlog query 混成一层
- 不让 audit replay 变成伪查询系统
- 不把场景专属对象粗暴塞进通用状态机

## 5. 已经验证的事实

- 当前 `TrackedTaskState` 已经是稳定任务级读模型入口
- 当前 file / SQLite repository 都直接持久化整个任务状态 JSON，适合先加 task-scoped read model
- 当前 `tasks` 表和 audit filter 都不适合回答 owner backlog / overdue queue 这类跨任务问题

## 6. 这次做对了什么

这次做对的地方，是没有一上来就定义一批 follow-up API，而是先把问题收紧成三层：

- task-scoped follow-up view
- cross-task queue query
- SLA monitoring view

这样后续不论是写代码，还是继续做交接 receipt 和告警 cluster，都更容易保持边界清楚。

## 7. 这一步如何真正产生影响

这份 direction note 的真正价值，在于它让 follow-up / SLA 第一次从“抽象对象层”进入“平台读层”。

这会直接影响后续路线：

- `tasks/{task_id}` 会更像正式协同对象入口
- 聚合 query 会更自然地从 dedicated projection 出发
- 高频场景之间会开始共享读模型，而不是共享文案
