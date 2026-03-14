# FA Experience Command Center Direction

## 1. 为什么现在做 UI

FA 当前已经有真实的生命周期、治理、审计、evidence 和跨任务监控 API，但在体验层仍主要依赖 `curl`。这会带来三个直接问题：

1. 价值不可见。潜在客户无法在几分钟内感知平台是否真能承接高价值制造协同。
2. 角色不可见。班组长、值班工程师、厂长和企业 IT 不会天然把 JSON 想象成他们的工作台。
3. AI 不可信。没有明确的 AI 标识、治理边界、证据面板和动作入口，系统更像一个技术 demo，而不是可采购产品。

因此，FA 下一阶段必须把“命令行可验证”升级成“浏览器中可理解、可演示、可操作”的产品体验。

## 2. 外部研究提炼

本方向主要参考以下公开设计系统与 AI 产品设计原则：

- Microsoft Fluent 2：强调清晰、专注、平静、层次和有意义的 motion，而不是过度装饰。
- IBM Carbon：强调面向工作流和数据密集场景的 productive UI，同时要求 AI presence 明确可识别。
- Atlassian AI / Rovo：强调 AI 必须被清楚标注、可编辑、可撤回，并且用户始终保有控制权。

对 FA 的直接结论：

1. 企业级体验不能做成玩具式“聊天框优先”，而要做成 command center / workbench。
2. Agentic 感要来自显式 AI 状态、生成动作、建议理由、证据和治理，而不是单纯炫技。
3. 渐变、玻璃、像素细节可以用，但只能服务于“方向感、科技感、层级感”，不能牺牲数据可读性。
4. 高价值页面应优先展示跨任务监控、运营队列、单任务 dossier 和一键 demo workflow。

## 3. 目标体验风格

目标风格定义：

- 视觉基调：欧美大型企业能接受的 executive dashboard + industrial control surface
- 结构气质：信息密度高，但层次清晰，不做消费级卡通化
- Agentic 识别：AI badge、生成式动作入口、任务级 evidence 与 audit timeline 显式可见
- 高科技语言：冷色渐变、毛玻璃、体积感、高亮像素角标、数字化网格背景
- 信任表达：任何 workflow action 都必须贴着 task status、approval role、evidence 和 audit

## 4. 体验信息架构

### 4.1 顶层结构

第一屏不做聊天，而做四层结构：

1. Executive hero
2. Platform pulse
3. Monitoring + queue workbench
4. Task dossier

### 4.2 关键模块

- Executive hero：一句话说明 FA 是制造 Agentic AI command surface，而不是接口集合
- Platform pulse：平台模式、系统层、delivery track、参考企业规模
- Monitoring：`follow-up`、`handoff receipt`、`alert cluster` 三类高频运营监控
- Queue workbench：直接暴露值班问题，而不是要求用户记 API
- Task dossier：plan、follow-up、receipt、alert cluster、evidence、governance、audit 一页收口
- Quick launch：一键灌入 demo workflow，避免空白页

## 5. 当前实现决策

本轮不新起独立前端仓库，先在现有 `fa-server` 内直接交付第一版体验层。原因：

1. 现有系统仍处于快速演进期，先把体验和 API 契约收紧，比先做重前端工程更重要。
2. 浏览器入口、聚合接口和 workflow action 一旦成立，后续再拆 React / design system 也更有边界。
3. Rust/Axum 直接托管首版静态体验层，能最快把产品从“接口型项目”变成“可演示平台”。

## 6. 分阶段计划

### Phase 1: Command Center Shell

目标：让用户打开根路径就能理解 FA。

范围：

- 根路径 `/` 提供浏览器 UI
- `/api/v1/experience/overview` 聚合平台、监控和队列预览
- Quick launch 直接创建高价值 demo workflow
- 任务级 dossier 支持查看治理、证据、审计和任务状态
- 首批 workflow action 支持 approve / execute / complete / follow-up accept / handoff acknowledge / escalate

退出标准：

- 无需 `curl` 也能完成首轮产品演示
- 监控、队列和任务 dossier 能由真实 API 驱动

### Phase 2: Operator Workbench

目标：让角色化操作真正进入页面。

范围：

- 过滤、排序、搜索、saved views
- 队列细分为 supervisor / maintenance / quality 三条工作台
- 风险与 SLA 可视化
- 更完整的 workflow action bar 与批量操作

### Phase 3: Enterprise Productization

目标：让 FA 从 demo 工作台演进成企业采购可评估产品。

范围：

- 认证、权限、角色化导航
- 实时刷新 / SSE / websocket
- 指标图表、值班看板、班次视图
- 设计 token、组件库、Figma 对齐
- 专门的 `executive`, `operator`, `governance` 页面体系

## 7. 当前切口完成定义

本轮开始的第一批代码，应至少完成：

- 浏览器入口成立
- 监控和队列可视化
- demo workflow 可从 UI 触发
- task dossier 可读
- 至少部分 workflow action 可从 UI 直接完成

## 8. 参考来源

- Fluent 2 design system: https://fluent2.microsoft.design/
- Material motion guidance: https://m3.material.io/styles/motion/overview
- IBM Carbon design system: https://carbondesignsystem.com/
- IBM Carbon AI presence guidance: https://carbondesignsystem.com/patterns/ai-presence/overview/
- Atlassian AI design guidance: https://atlassian.design/components/ai
