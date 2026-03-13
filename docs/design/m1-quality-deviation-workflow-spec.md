# M1 Quality Deviation Candidate Workflow Specification

## 1. Workflow 名称

`质量偏差隔离与处置建议`

## 2. 文档定位

本文件不是当前 `v0.2.0` 已打通的正式 pilot workflow，而是当前 pilot 之后最值得推进的第二条候选 workflow specification baseline。

它的作用是：

- 让平台从设备异常场景扩展到质量异常场景
- 提前冻结质量协同所需的角色、证据、审批和回退边界
- 为后续 `QMS / MES / ERP` connector 投资和治理模型扩展提供输入

## 3. 目的

当生产过程出现超规、偏差、异常批次或疑似质量风险时，平台需要帮助企业更快完成：

- 偏差受理
- 影响范围识别
- 隔离与复检建议
- 升级与审批协同
- 后续处置跟踪

目标不是让 AI 自动做质量放行或报废决策，而是把质量异常处理从“信息分散、跨部门沟通慢、责任边界模糊”推进到“证据驱动、责任清晰、可审计”的协同流程。

## 4. 触发条件

以下任一条件成立时，可触发该 workflow：

- QMS 中出现质量偏差或不合格记录
- MES 检测到某批次/工单的质量指标超出控制限
- 质量工程师或班组长主动上报疑似批量质量风险
- 客诉或终检异常回溯到当前生产批次

## 5. 参与角色

| 角色 | 职责 |
| --- | --- |
| Production Supervisor | 说明现场情况、停线影响和当前生产状态 |
| Quality Engineer | 主导偏差调查、复检建议和初步处置建议 |
| Quality Manager | 对高风险处置、批次隔离和升级动作承担审批责任 |
| Manufacturing Engineer | 评估工艺、设备、参数或工装对偏差的影响 |
| Logistics / Warehouse Coordinator | 执行物料、在制品或成品隔离动作 |
| FA Agent / Orchestrator | 汇聚证据、组织跨角色协同、生成建议与审计轨迹 |
| Delivery Owner / System Admin | 维护运行边界、回退策略和试运行开关 |

## 6. 涉及系统与资产

| 对象 | 当前阶段作用 |
| --- | --- |
| QMS | 提供偏差、检验、处置与 CAPA 上下文 |
| MES | 提供工单、设备、班次、工艺批次和产出上下文 |
| ERP | 提供订单、客户、库存和批次业务影响上下文 |
| LIMS / SPC | 提供检测数据、趋势和超限记录 |
| WMS | 提供批次库存位置与隔离对象信息 |
| FA Platform | 编排任务、审批、证据、审计与回放 |

## 7. 目标输出

workflow 的输出应包括：

- 偏差摘要
- 影响批次 / 工单 / 在制品范围
- 证据清单
- 隔离建议
- 复检建议
- 处置建议草稿
- 升级与审批路径
- 任务状态与审计轨迹

## 8. 流程步骤

### Step 1. 发起偏差任务

发起人通过任务 intake 提交质量异常，输入至少包含：

- 偏差描述
- 相关批次 / 工单 / 产品标识
- 已知影响范围
- 风险级别
- 期望结果

### Step 2. 读取质量与生产上下文

编排器读取：

- QMS 偏差与检验记录
- MES 生产批次与工艺上下文
- 必要时读取 ERP / WMS 的订单和库存上下文

并把所有读取动作写入审计。

### Step 3. 生成影响范围与初步建议

平台基于上下文生成：

- 偏差事件摘要
- 可能受影响的批次、工单或在制品范围
- 需要立即隔离的对象建议
- 需要复检或追加采样的对象建议
- 需要会签的角色建议

### Step 4. 人工审批与升级门控

当建议动作涉及：

- 批次隔离
- 出货冻结
- 返工 / 报废建议
- 升级 CAPA 或跨部门处置

则必须进入人工审批。

该场景的目标治理方向应是由 `Quality Manager` 或等价质量责任人承担正式审批责任。

### Step 5. 执行跟踪

审批通过后，任务进入执行态，由人工或外部系统完成真实动作。平台只记录：

- 隔离动作开始
- 复检或复核开始
- 处置结果
- 是否升级为 CAPA / NCR / 客诉闭环

### Step 6. 审计回放

任务结束后，平台应能通过：

- `/api/v1/tasks/{task_id}`
- `/api/v1/tasks/{task_id}/evidence`
- `/api/v1/tasks/{task_id}/governance`
- `/api/v1/tasks/{task_id}/audit-events`

回放整条质量偏差处理轨迹。

## 9. Agentic Pattern 映射

| 阶段 | Pattern | 原因 |
| --- | --- | --- |
| intake / planning | `coordinator` | 需要跨质量、生产、仓储和管理角色协调 |
| evidence gathering | `ReAct loop` | 需要围绕偏差证据逐步形成影响分析和建议 |
| approval | `human-in-the-loop` | 批次隔离、处置和升级不能由模型直接裁决 |
| downstream action | `deterministic workflow + custom business logic` | 真实 QMS / ERP / WMS 写入必须被策略封装 |

## 10. 证据与审计要求

必须记录：

- 任务发起人
- 偏差编号或质量事件标识
- 受影响批次 / 工单 / 产品标识
- 每次 connector 读取
- 审批请求与审批决定
- 执行说明、升级说明和最终处置说明

候选证据源包括：

- QMS 偏差记录
- MES 批次与工艺上下文
- SPC / LIMS 检测记录
- ERP / WMS 订单与库存影响信息
- 人工补充说明

## 11. 明确禁止的动作

当前和后续试运行阶段都应明确禁止：

- 自动做质量放行决定
- 自动做报废决定
- 自动冻结或释放真实库存
- 自动关闭 CAPA / NCR
- 在没有人工批准的前提下写入真实 QMS / ERP / WMS

## 12. 回退策略

若 workflow 在任何节点出现异常，应回退到人工流程：

1. 记录失败原因与当前证据
2. 保留任务、审批和审计轨迹
3. 由质量负责人接手处置
4. 必要时创建后续调查任务或 CAPA 任务继续跟踪

## 13. 与当前平台能力对齐

当前已支持的通用能力：

- 任务 intake
- 任务查询
- evidence snapshot
- governance 输出
- 审批与修订重提
- 执行开始 / 完成 / 失败
- 审计过滤与任务回放
- 文件 / SQLite / 内存三种本地运行模式

当前尚未支持但该场景后续必须补齐：

- `QMS / ERP / WMS / LIMS` connector baseline
- lot / batch / material 级 evidence 建模
- `Quality Manager` 级审批策略与角色映射
- 质量处置 taxonomy，例如 `hold / rework / scrap / use-as-is`
- CAPA / NCR / deviation case draft 输出
- 质量影响范围的结构化表达

## 14. 试运行验收标准

进入后续阶段前，至少满足：

1. 能完成 1 次端到端质量偏差任务闭环演示。
2. 能清楚说明受影响范围、隔离建议和升级理由。
3. 所有关键节点具备审计记录。
4. 质量角色能看懂证据来源、责任边界和回退路径。
5. 不发生越权自动放行、自动报废或自动冻结。
