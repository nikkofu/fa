# M1 Shift Handoff Candidate Workflow Specification

## 1. Workflow 名称

`班次交接摘要与待办提取`

## 2. 文档定位

本文件不是当前 `v0.2.0` 已打通的正式 pilot workflow，而是高频、低风险、最适合快速扩面的日常协同 workflow specification baseline。

它的作用是：

- 把平台从“只服务异常处理”扩展到“服务日常运营协同”
- 明确交接场景下的证据、follow-up、SLA 和责任边界
- 为后续 `shift log / incident log / task history / Andon` 类输入建设提供设计基线

## 3. 目的

当班次即将结束或交接发生时，平台需要帮助现场角色更快完成：

- 本班关键事件摘要
- 未关闭异常与阻塞项识别
- follow-up 待办提取
- owner / 下一班组路由建议
- 风险提醒与升级建议

目标不是代替班组长完成责任交接，而是把“口头交接、纸面补充、信息散落”推进到“摘要清晰、待办明确、风险可追踪、责任可审计”的日常协同流程。

## 4. 触发条件

以下任一条件成立时，可触发该 workflow：

- 班次结束前固定时间窗口触发交接准备
- 班组长主动发起交接摘要任务
- 交接会前需要快速汇总本班异常、停机、待处理项
- 下一班组需要查看上班遗留问题和风险重点

## 5. 参与角色

| 角色 | 职责 |
| --- | --- |
| Production Supervisor | 对本班关键事件、遗留问题和交接重点负责确认 |
| Shift Lead / Team Lead | 补充班组执行状态、资源约束和待办 owner |
| Incoming Shift Supervisor | 接收交接摘要，确认下一班跟进重点 |
| Maintenance Engineer | 对未关闭设备异常提供状态说明或 follow-up 输入 |
| Quality Engineer | 对质量相关遗留风险提供补充说明或升级建议 |
| FA Agent / Orchestrator | 汇总事件、提取待办、生成摘要、标记风险与审计轨迹 |
| Delivery Owner / System Admin | 维护模板边界、回退策略和运行开关 |

## 6. 涉及系统与资产

| 对象 | 当前阶段作用 |
| --- | --- |
| MES | 提供工单、产量、异常、停机和线体上下文 |
| Shift Log / Incident Log | 提供交接原始记录、备注和未关闭事项 |
| Task History / Audit | 提供已有任务、审批结果和 follow-up 线索 |
| CMMS | 提供设备异常或未关闭维护事项上下文 |
| FA Platform | 编排任务、摘要、待办提取、证据、审计与回放 |

## 7. 目标输出

workflow 的输出应包括：

- 本班摘要
- 关键异常与阻塞项清单
- 未关闭 follow-up 清单
- 建议 owner / 下一步 / 期望时间窗口
- 风险升级提醒
- 交接任务状态与审计轨迹

## 8. 流程步骤

### Step 1. 发起交接摘要任务

发起人通过任务 intake 提交班次交接请求，输入至少包含：

- 班次标识
- 线体 / 区域 / 工段范围
- 交接时间窗口
- 是否存在重点风险提示
- 期望输出对象，例如下一班组或值班主管

### Step 2. 读取班次与遗留事项上下文

编排器读取：

- MES 班次事件与异常上下文
- shift log / incident log 中的交接备注
- 当前未完成 task / audit 线索
- 必要时读取 CMMS 中未关闭设备事项

并把所有读取动作写入审计。

### Step 3. 生成摘要与待办提取

平台基于上下文生成：

- 本班核心事件摘要
- 关键风险和阻塞项排序
- 待跟进事项列表
- 建议 owner、建议 next step 和建议时限
- 需要升级为正式任务的候选项

### Step 4. 人工确认与必要升级

交接摘要默认是 assistive / coordination 输出，不直接产生真实写动作。

若平台识别到以下情况，应提醒人工确认或升级：

- 高风险设备异常仍未关闭
- 涉及质量疑虑但未形成正式偏差任务
- follow-up 已超出正常交接范围，需要新建正式任务
- 下一班组无法单独承接，需要主管或跨部门介入

当前阶段的目标治理方向是：

- 交接摘要本身不要求正式审批
- 由 `Production Supervisor` 或等价班次负责人确认交接内容
- 任何高风险 follow-up 的真实任务化或升级动作，仍应回到已有 governed workflow 主链

### Step 5. 执行跟踪

交接完成后，平台只记录：

- 摘要是否被确认
- 哪些 follow-up 被标记为遗留事项
- 哪些项被转入正式任务或升级跟踪
- 是否存在未接收或信息不足的交接风险

### Step 6. 审计回放

任务结束后，平台应能通过：

- `/api/v1/tasks/{task_id}`
- `/api/v1/tasks/{task_id}/evidence`
- `/api/v1/tasks/{task_id}/governance`
- `/api/v1/tasks/{task_id}/audit-events`

回放整条交接摘要与待办提取轨迹。

## 9. Agentic Pattern 映射

| 阶段 | Pattern | 原因 |
| --- | --- | --- |
| intake / summarization | `single-agent` | 单条交接摘要本身以汇总和提取为主，风险较低 |
| cross-system context assembly | `coordinator` | 需要汇总 MES、shift log、task history 等多源上下文 |
| follow-up extraction | `ReAct loop` | 需要围绕事件线索判断哪些项应进入遗留清单或升级候选 |
| downstream escalation | `deterministic workflow + custom business logic` | 真实任务化、升级或后续写入必须受规则约束 |

## 10. 证据与审计要求

必须记录：

- 任务发起人
- 班次标识和交接窗口
- 每次 connector 读取
- 被纳入摘要的关键事件引用
- 每条 follow-up 的来源说明
- 人工确认说明或升级说明

候选证据源包括：

- MES 班次事件摘要
- shift log / incident log 原始记录
- task history / audit 线索
- CMMS 未关闭事项
- 人工补充说明

## 11. 明确禁止的动作

当前和后续试运行阶段都应明确禁止：

- 自动替现场负责人确认交接完成
- 自动关闭遗留任务
- 自动重新分配真实 owner
- 自动创建真实高风险任务而不经过人工确认
- 自动修改 MES / CMMS / shift log 中的真实业务记录

## 12. 回退策略

若 workflow 在任何节点出现异常，应回退到人工交接流程：

1. 记录失败原因与当前已汇总证据
2. 保留任务、审计和摘要草稿
3. 由班组负责人手动完成交接
4. 必要时创建后续任务补录遗漏信息

## 13. 与当前平台能力对齐

当前已支持的通用能力：

- 任务 intake
- 任务查询
- evidence snapshot
- governance 输出
- 审计过滤与任务回放
- 审批与修订重提
- 执行开始 / 完成 / 失败
- 文件 / SQLite / 内存三种本地运行模式

当前尚未支持但该场景后续必须补齐：

- shift log / incident log connector baseline
- follow-up / owner / due date / SLA 的结构化建模
- 时间线型 evidence 表达，而不只是离散 snapshot
- 跨任务聚合视图
- 交接摘要确认与接收状态的正式对象模型

## 14. 试运行验收标准

进入后续阶段前，至少满足：

1. 能完成 1 次端到端交接摘要任务演示。
2. 能清楚列出本班重点、遗留 follow-up 和风险提醒。
3. 所有关键读取和摘要生成节点具备审计记录。
4. 班组角色能看懂摘要、待办和升级边界。
5. 不发生越权自动关单、自动分配或自动升级高风险动作。
