# Alert Triage Cluster Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

告警场景最容易被做坏的一点，是很多系统会把“收到很多信号”直接等同于“应该创建很多任务”。但原始信号、异常簇、分诊草稿和后续待办根本不是一回事。

如果 cluster / ingestion 语义不单独做，告警场景就永远只有噪音，没有稳定协同对象。

## 2. 这一阶段的创新点

这一阶段的关键，是把告警主线拆成了四层：

- raw alert event
- alert cluster
- triage draft
- follow-up item

这样之后，平台第一次能清楚回答“哪条信号为什么会变成哪个异常簇，又为什么会继续变成后续动作”。

## 3. 这如何改变世界

制造现场很多告警协同损耗，并不是因为没有信号，而是因为没人能稳定说明：

- 哪些信号其实是一回事
- 哪些应该被抑制
- 哪些已经形成需要跟进的异常簇
- 哪些簇才真的值得进入任务和升级

cluster / ingestion 对象真正改变的，就是这层事件协同秩序。

## 4. 对自己的要求

- 不把 raw alert 直接写成 triage task
- 不把 cluster 和 follow-up 混成一个状态机
- 不让 event-driven 成为放松治理边界的借口

## 5. 已经验证的事实

- 当前 task detail 是最自然的第一层 cluster draft 挂载点
- 当前 evidence 可以承接 cluster draft JSON，但不能替代 cluster query
- 当前 audit 可以回放 cluster 变化，但不能替代 cluster projection 本身

## 6. 这次做对了什么

这次做对的地方，是没有继续把告警问题都塞给 connector 或 follow-up，而是承认告警主线里还有一层专属的聚合对象和受控入口边界。

这样后续不论是做 `Scada / Andon` mock baseline，还是做 event ingestion，都更容易保持边界清楚。

## 7. 这一步如何真正产生影响

这份 direction note 的真正价值，在于它让 `产线告警聚合与异常分诊` 从“有分诊建议”继续推进到“有稳定 cluster 对象和受控事件入口”。

这会直接影响后续路线：

- 告警 task detail 会更完整
- cluster backlog 会变成真实的运行视图
- 高频事件协同会开始具备区别于普通任务 intake 的产品深度
