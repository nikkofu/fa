# Alert Triage Alignment Stage Notes - 2026-03-12

## 1. 为什么告警场景现在要补对齐清单

告警分诊最容易被误解成“把告警总结一下”。但只要真的进入产品，它立刻会暴露出更硬的问题：信号从哪里来、哪些告警属于同一簇、哪些只是提醒、哪些必须升级、升级之后谁跟、多久必须响应。

如果这些问题不提前收紧，告警场景就会永远停留在高时效摘要 demo。

## 2. 这一阶段的创新点

这一阶段的关键，是把告警场景从 workflow spec 推到实现对齐层。

这意味着平台开始明确：

- 哪些输入必须通过 connector 和事件边界来承接
- 哪些输出应进入 alert cluster、triage label 和 escalation candidate
- 哪些风险必须继续回到正式 governed workflow

## 3. 这如何改变世界

制造业并不缺告警系统，真正缺的是一层能把多源告警变成可理解、可归并、可升级、可追踪协同对象的操作层。

把告警场景做成 alignment checklist，本质上是在把“有信号”推进到“有协同”。

## 4. 对自己的要求

- 不把 event-driven 误写成默认自动化控制
- 不把当前不存在的 `Scada / Andon / incident log` baseline 写成已具备能力
- 不把 triage confirmation 写成 formal approval

## 5. 已经验证的事实

- 当前平台已经有足够主链承接告警任务：task、evidence、governance、audit
- 当前平台没有告警真正需要的输入和对象层：默认 `Scada` connector、`Andon / incident log` target、alert cluster、timeline evidence
- 告警场景真正缺的不是更多 endpoint，而是输入对象、聚合对象和升级边界

## 6. 这次做对了什么

这次做对的地方，是没有继续往“更聪明的告警总结”方向走，而是把问题拉回 connector、evidence、governance、follow-up 和 event-ingestion 这些更硬的工程问题上。

这样后续无论是做 follow-up / SLA read model，还是做 shift handoff receipt，都能落在同一条协同主线上。

## 7. 这一步如何真正产生影响

这份 alignment checklist 的真正价值，在于它让告警场景第一次拥有了明确的实现顺序：

- 先补 mock 告警输入
- 再冻结 alert cluster / triage draft
- 再收敛 follow-up / SLA 和 event-ingestion
- 最后进入跨时间窗聚合查询

这会直接提升平台路线的清晰度，也会让高频事件协同不再只是一个设计口号。
