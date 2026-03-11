# Team Operating Model

## 1. 目的

本文件定义 FA 项目的团队组织方式和协作规则，确保项目在 0-1 阶段同时满足两类要求：

- 软件工程上可持续开发、测试、发布和维护。
- PMP 视角下可做范围、进度、风险、沟通、质量和变更控制。

## 2. 团队拓扑

### 2.1 Core team

| 角色 | 核心职责 | 主要产出 |
| --- | --- | --- |
| Project Sponsor | 预算、业务优先级、关键升级裁决 | 目标确认、资源批准、阶段 Go/No-Go |
| Product Owner | 业务需求、价值排序、验收标准 | Backlog、验收标准、UAT 签收 |
| Delivery Lead / PMP | 节奏、范围、里程碑、RAID、状态汇报 | 计划、周报、风险日志、变更记录 |
| Solution Architect / Tech Lead | 技术路线、架构边界、关键设计决策 | 架构设计、ADR、技术拆分 |
| Engineering Lead | 开发执行、代码质量、分工与交付质量 | 技术任务拆解、PR 审核、交付达成 |
| QA Lead | 测试策略、测试覆盖、UAT 准备、缺陷分级 | 测试计划、测试报告、发布建议 |
| Pilot Plant Owner | 试运行环境、现场资源、SOP 约束与验收 | Pilot 场景、试运行批准、反馈 |

### 2.2 Extended team

| 角色 | 参与场景 |
| --- | --- |
| Domain SME | 工艺、设备、班组流程、质量与安全规则确认 |
| Data / Integration Engineer | ERP、MES、CMMS、SCADA、QMS 接口对接 |
| Security / Compliance Reviewer | 权限、审计、隐私与外部模型接入边界 |
| DevOps / Platform Engineer | 环境、部署、观测、运行变更控制 |
| UX / Frontend | 操作台、审批台、试运行界面与交互逻辑 |

## 3. 决策权分层

### 3.1 业务决策

- 由 `Project Sponsor + Product Owner` 负责最终裁定。
- 包括范围优先级、里程碑目标、试运行场景、业务接受度。

### 3.2 技术决策

- 由 `Solution Architect / Tech Lead` 负责最终裁定。
- 必须记录到 ADR 或架构文档。
- 涉及供应商接入、模型边界、设备安全策略时，需要拉上 Domain SME 或 Security Reviewer。

### 3.3 发布决策

- 由 `Engineering Lead + QA Lead + Delivery Lead` 联合出具建议。
- 涉及试运行上线时，必须增加 `Pilot Plant Owner` 批准。

### 3.4 范围和变更决策

- 小变更由 `Product Owner + Delivery Lead` 处理。
- 影响基线计划、试运行窗口、关键架构或成本的变更，升级到 CCB。

## 4. RACI

| 工作项 / 产物 | Sponsor | PO | Delivery/PMP | Architect | Eng Lead | QA Lead | Pilot Owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 项目章程 | A | C | R | C | I | I | C |
| 路线图与里程碑 | A | R | R | C | C | C | C |
| Backlog 优先级 | C | A/R | C | C | C | C | C |
| WBS / Sprint 计划 | I | C | A/R | C | R | C | I |
| 架构方案 / ADR | I | C | I | A/R | C | C | I |
| 开发任务拆分 | I | C | C | C | A/R | C | I |
| 测试策略与计划 | I | C | C | C | C | A/R | C |
| UAT 验收标准 | C | A/R | C | C | C | R | C |
| 发布决策 | I | C | R | C | A/R | A/R | C |
| 试运行批准 | A | C | R | C | C | C | A/R |
| 风险日志 / RAID | I | C | A/R | C | C | C | C |
| 变更请求评审 | C | C | A/R | C | C | C | C |
| 版本说明 / Release note | I | C | R | I | C | C | I |

说明：

- `A` = Accountable
- `R` = Responsible
- `C` = Consulted
- `I` = Informed

## 5. 工作协作原则

### 5.1 单一事实源

- Backlog、风险、变更、决策、交付文档都以 GitHub 仓库和仓库文档为准。
- 会议纪要如影响范围、计划或责任分配，必须回写到仓库。

### 5.2 所有工作必须挂靠工单

- Feature、defect、risk、change request 都必须有 issue。
- PR 必须关联 issue。
- 没有关联 issue 的工作默认不计入项目进展。

### 5.3 文档与代码同步演进

- 新能力涉及架构边界变更时必须更新 ADR 或架构文档。
- 新工作流涉及职责、发布、试运行方式变化时必须更新治理文档。

### 5.4 明确阻塞升级路径

- 阻塞超过 24 小时必须在 standup 或项目频道中显式升级。
- 阻塞涉及外部系统、现场资源或供应商配合时，由 Delivery Lead 负责推动闭环。

### 5.5 高风险改动先治理后编码

- 涉及设备动作、审批链、业务写入、权限、审计策略的需求，先完成设计和评审，再进入开发。

## 6. 会议与沟通节奏

### 6.1 Daily standup

- 频率：每工作日
- 时长：15 分钟
- 参与：Core team
- 输出：
  - 昨日完成
  - 今日计划
  - 阻塞项
  - 风险升级

### 6.2 Weekly planning and commitment

- 频率：每周一
- 时长：45 至 60 分钟
- 主持：Delivery Lead
- 目标：
  - 回顾上周完成情况
  - 确认本周承诺项
  - 调整短期依赖与资源

### 6.3 Weekly product and backlog review

- 频率：每周一次
- 参与：PO、Architect、Eng Lead、QA Lead、Domain SME
- 输出：
  - Backlog 优先级调整
  - 新需求是否进入 Discovery
  - 现有需求是否满足 Definition of Ready

### 6.4 Architecture and design review

- 频率：每周一次或按需
- 主持：Architect
- 触发条件：
  - 新集成接口
  - 新 agent 模式
  - 新审批策略
  - 新运行边界
- 输出：
  - 设计结论
  - ADR 或设计文档更新

### 6.5 Quality and release readiness review

- 频率：每周一次，发布周可增加
- 主持：QA Lead
- 输出：
  - 当前缺陷状态
  - 测试覆盖缺口
  - 发布建议
  - 回退准备情况

### 6.6 RAID and change control review

- 频率：每周一次
- 主持：Delivery Lead
- 输出：
  - 风险更新
  - 依赖升级
  - 变更请求处理结果
  - 决策待办

### 6.7 Steering committee

- 频率：每两周或每月一次
- 参与：Sponsor、PO、Delivery Lead、Architect、Pilot Owner
- 输出：
  - 里程碑状态
  - 预算与资源确认
  - 重大范围或策略裁决

## 7. 沟通通道与响应时限

| 类型 | 主通道 | 响应时限 | 责任人 |
| --- | --- | --- | --- |
| 日常协作 | GitHub issue / PR + 项目频道 | 1 个工作日内 | 对应责任人 |
| 阻塞升级 | 项目频道 + standup | 24 小时内升级 | 任务 owner |
| 关键风险 | RAID review / 项目频道 | 当天升级 | Delivery Lead |
| 发布异常 | War room / issue / 回退通道 | 30 分钟内响应 | Eng Lead + QA Lead |
| 试运行事件 | Pilot 通道 + Incident issue | 15 分钟内确认 | Pilot Owner + Eng Lead |

## 8. 交付产物与责任

| 产物 | 默认位置 | 主负责人 |
| --- | --- | --- |
| Charter | `docs/project-charter.md` | Delivery Lead |
| Roadmap | `docs/roadmap.md` | Product Owner + Delivery Lead |
| Architecture | `docs/architecture.md` | Architect |
| ADR | `docs/adr/` | Architect |
| Governance docs | `docs/governance/` | Delivery Lead + Architect |
| Release note / Changelog | `CHANGELOG.md` | Eng Lead + Delivery Lead |
| 测试计划 / UAT | 后续 `docs/qa/` | QA Lead |
| 试运行手册 | 后续 `docs/pilot/` | Pilot Owner + Delivery Lead |

## 9. 团队工作约定

### 9.1 Branch and PR discipline

- 默认在 `main` 上保持可发布状态。
- 所有开发通过 feature branch 提交。
- PR 合并前必须通过 CI。
- 至少 1 位工程负责人审核通过。
- 涉及架构变更时，Architect 必须参与 review。

### 9.2 Estimation and commitment

- Epic / capability 使用 T-shirt size 或 milestone 级估算。
- Sprint / 周计划内任务使用 ideal day 或 story point。
- 承诺以团队可交付能力为边界，不以乐观预期替代计划。

### 9.3 Documentation discipline

- 任何影响交付、治理、架构的改动都要同步更新文档。
- 文档缺失可阻止需求进入“Ready for Dev”。

### 9.4 Meeting discipline

- 每个例会都必须有 owner、输入、输出、会后动作。
- 没有输出的会议视为无效会议。

## 10. 当前执行建议

在当前 0-1 阶段，建议以“小核心团队 + 强文档治理 + 周交付”为主，不要过早复制大公司的复杂流程。流程必须足够严谨，但仍服务于试运行落地速度。
