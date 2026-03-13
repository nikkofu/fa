# High-frequency Agentic Function Priority Map

## 1. 文档目的

本文件用于把制造业中的 Agentic AI 探索，从“哪些 workflow 值得做”进一步推进到“哪些高频、高优先级功能最值得先产品化”。

workflow 解决的是一条业务路径如何闭环，function 解决的是平台最应该先把哪些能力打磨成可复用能力包。

本文件回答 4 个问题：

1. 制造企业里哪些 Agentic 功能最常用、最紧急。
2. 哪些功能最适合作为当前平台的优先产品化方向。
3. 这些功能分别属于 `L1 / L2 / 受控 L3` 中的哪一层。
4. 哪些功能应该先变成平台通用能力，哪些再进入单条 workflow spec。

## 2. 判断标准

把功能而不是 workflow 排优先级时，建议重点看 6 个维度：

| 维度 | 说明 |
| --- | --- |
| 触发频率 | 每班、每日、每周都会发生，还是低频事件 |
| 响应紧迫性 | 是否需要分钟级、小时级响应 |
| 适用面 | 能否跨产线、班组、工厂复用 |
| 当前平台匹配度 | 是否能复用现有 task lifecycle、evidence、governance、audit |
| 写操作风险 | 是否只读、建议、还是逼近真实执行 |
| 组织接受度 | 现场是否容易接受 AI 先从这个功能开始参与 |

## 3. 高优先级功能候选池

### 3.1 Top 8 功能

| 功能 | 典型触发 | 主要价值 | 频率 | 紧迫性 | 治理层级 | 当前适配度 |
| --- | --- | --- | --- | --- | --- | --- |
| 班次交接摘要与待办提取 | 每班次交接 | 降低遗漏，统一待办与风险交接 | 很高 | 中 | L1 | 很高 |
| 产线告警聚合与异常分诊 | 实时告警流 | 降噪、去重、分级、定责 | 很高 | 很高 | L1/L2 | 高 |
| 生产中断事件协调 | 停机、堵料、换线异常 | 缩短跨角色协调时间 | 高 | 很高 | L2 | 高 |
| 重复异常聚类与复盘提示 | 每日 / 每周异常复盘 | 减少同类问题重复处理 | 高 | 中 | L1/L2 | 高 |
| 跟进项 owner 路由与 SLA 追踪 | 交接、例会、异常关闭 | 防止任务漂移和超时 | 很高 | 中高 | L2 | 很高 |
| 质量异常预分诊与证据包组装 | 偏差、超规、客诉前置信号 | 加速质量团队进入判断 | 中高 | 高 | L2 | 中高 |
| 关键物料短缺协同建议 | 缺料、来料延误 | 保护产线节拍与排程稳定 | 高 | 高 | L2 | 中高 |
| 日损失热点与优先级排序 | 每日生产复盘 | 让管理层快速看见最应处理的问题 | 很高 | 中 | L1 | 高 |

### 3.2 为什么这些功能比“更宏大闭环”更优先

这些功能共同具备 5 个特点：

- 触发频率高，能快速建立使用习惯
- 大部分以只读、建议和协同为主
- 容易和现有 `evidence / governance / audit` 主链对齐
- 不需要一开始就进入真实写操作
- 能先证明平台不是“只会处理罕见大事故”，而是能进入日常经营与现场节奏

## 4. 最值得优先产品化的功能包

### 4.1 Function Pack A: Shift Handoff Copilot

核心功能：

- 班次事件摘要
- 未关闭异常提取
- 风险与阻塞项排序
- 待跟进 owner 和 SLA 建议

为什么优先：

- 高频到几乎每班都会发生
- 风险低，容易被一线接受
- 需要的系统主要是 `MES + shift log + task history`
- 与当前 `task / evidence / audit` 能力高度匹配

建议模式：

- `single-agent` 负责总结
- `coordinator` 负责跨系统信息汇总
- `deterministic workflow` 负责交接模板和输出格式边界

当前基线文档：

- [m1-shift-handoff-workflow-spec.md](../design/m1-shift-handoff-workflow-spec.md)
- [m1-shift-handoff-alignment-checklist.md](../design/m1-shift-handoff-alignment-checklist.md)

### 4.2 Function Pack B: Alert Triage Copilot

核心功能：

- 告警聚合与去重
- 告警分级与初步路由
- 建议是否升级成人工处理任务
- 形成告警证据包与短摘要

为什么优先：

- 告警流是制造现场最典型的高频输入之一
- 价值不仅在“总结”，更在降低噪音和减少误升级
- 适合把平台从静态任务 intake 推进到“事件驱动任务生成”

建议模式：

- `single-agent` 用于低风险告警归类
- `coordinator` 用于跨 `MES / SCADA / Andon` 证据汇总
- `human-in-the-loop` 只在需要停线、质量升级或高风险排查时触发

当前基线文档：

- [m1-alert-triage-workflow-spec.md](../design/m1-alert-triage-workflow-spec.md)

### 4.3 Function Pack C: Follow-up Tracker

核心功能：

- 从交接、异常、会议和审批结果里提取 follow-up
- 自动识别 owner、due date、blocked status
- 按超时或依赖阻塞触发提醒或升级建议

为什么优先：

- 这类功能横跨几乎所有 workflow
- 平台一旦没有 follow-up 管理，就很容易退回“会总结但不闭环”
- 与现有 task lifecycle 和 audit 主链天然一致

建议模式：

- `coordinator` 负责跨任务聚合
- `deterministic workflow` 负责 SLA 计算和升级规则

当前基线文档：

- [m1-follow-up-sla-model-alignment-checklist.md](../design/m1-follow-up-sla-model-alignment-checklist.md)

### 4.4 Function Pack D: Repeated Anomaly Memory

核心功能：

- 相似异常聚类
- 历史处理路径回看
- 常见根因提示
- 建议是否复用已有处理模板

为什么优先：

- 设备、质量、物料问题都存在重复发生现象
- 价值来自减少重复问询和重复调查
- 对制造组织的“经验沉淀”价值高

建议模式：

- `ReAct loop` 负责围绕历史证据形成解释
- `L1/L2` 为主，避免直接推动高风险动作

## 5. 推荐优先级排序

如果按“高频 + 高优先级 + 易落地”综合排序，建议顺序如下：

1. `班次交接摘要与待办提取`
2. `产线告警聚合与异常分诊`
3. `跟进项 owner 路由与 SLA 追踪`
4. `重复异常聚类与复盘提示`
5. `生产中断事件协调`
6. `质量异常预分诊与证据包组装`
7. `关键物料短缺协同建议`
8. `日损失热点与优先级排序`

排序理由：

- 前 4 项既高频，又最适合先做成平台通用能力。
- 第 5 到 7 项更偏 governed coordination，价值高，但对 connector 和治理要求更重。
- 第 8 项虽高频，但更偏管理视角总结，优先级略低于现场即时协同功能。

## 6. 功能与当前平台能力的对齐

### 6.1 当前已经具备的基础

这些高频功能可以直接复用当前平台的以下能力：

- `tasks/intake` 与 `tasks/plan`
- `task status lifecycle`
- `evidence snapshot`
- `governance` 输出
- `audit replay`
- `correlation_id`
- mock `MES / CMMS` read-only connector

### 6.2 当前最缺的通用能力

如果想把高频功能做成真正的平台 feature，而不只是文档想法，当前最缺的是：

1. 事件驱动输入，而不只是人工任务 intake。
2. 时间线型 evidence 表达，而不只是离散 record snapshot。
3. owner / due date / SLA / blocked status 的结构化字段。
4. 告警、交接、follow-up 这类“日常协同对象”的正式模型。
5. 跨任务聚合视图，而不只是单任务查询。

## 7. 对产品路线的启发

### 7.1 平台不应只围绕“高风险审批”建设

如果平台只强化审批、治理和高风险异常，就会出现一个问题：

- 能处理重要但低频的事
- 却无法进入每天都在发生的工作节奏

真正能建立组织使用习惯的，往往是高频、低到中风险、重复发生的协同功能。

### 7.2 下一阶段最有价值的是把最小读模型推进到代码切口

接下来更值得推进的，不是再选一个“更宏大的跨系统闭环”，而是：

1. 开始评估最小 `follow_up_items` 读模型进入 `tasks/{task_id}` 的实现切口（已完成）。
2. 开始评估最小 `handoff_receipt` 读模型进入交接任务详情的实现切口（已完成）。
3. 开始评估最小 `alert_cluster_drafts` 读模型进入告警任务详情的实现切口（已完成）。

这三件事能让 FA 从“异常处理平台”更像“制造现场日常协同操作层”。

## 8. 推荐的后续动作

### 8.1 文档优先级

建议按以下顺序继续输出文档：

1. follow-up task read model implementation cut note（已完成）
2. shift handoff receipt task read model implementation cut note（已完成）
3. alert triage task read model implementation cut note（已完成）

### 8.2 连接器优先级

围绕这些高频功能，连接器投资顺序建议微调为：

1. `MES`
2. shift log / incident log
3. `SCADA / Andon`
4. `CMMS`
5. `QMS`
6. `ERP / WMS`

### 8.3 平台能力优先级

在代码层面，后续最值得优先化的能力是：

1. 事件到任务的受控转换
2. timeline-style evidence
3. follow-up / owner / due date / SLA 建模
4. 多任务聚合视图

## 9. 结论

从制造业真实使用节奏看，最值得优先推进的 Agentic 功能，并不全是最重、最敏感、最宏大的闭环。

更高频、更高优先级的功能，往往是这些：

- 交接摘要
- 告警分诊
- follow-up 追踪
- 重复异常记忆

它们更容易进入日常运营，也更适合先把 FA 打磨成一个真正会“协同工作”的平台，而不是只在重大异常时才被想起的系统。
