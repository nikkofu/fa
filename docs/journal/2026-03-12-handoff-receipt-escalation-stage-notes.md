# Handoff Receipt Escalation Stage Notes - 2026-03-12

## 1. 为什么这一层必须继续往前走

只有 acknowledgement 还不够，因为 `acknowledged_with_exceptions` 只能说明接收方指出了问题，不能说明系统已经知道谁来接住后续治理责任。

这一层单独拿出来，就是为了让 receipt 第一次拥有显式 escalation action。

## 2. 这一阶段的创新点

这一阶段的关键，是没有把 escalation 塞进审批链，也没有直接做 backlog / queue，而是先加一条最小显式 action：

- 只对 `shift handoff` receipt 生效
- 只允许 `acknowledged_with_exceptions -> escalated`
- 只允许发送侧 accountable 角色执行

这意味着平台开始明确：

- acknowledgement 与 escalation 是两条不同协同动作
- exception note 不再只是静态文本，而能进入可审计的治理动作
- task-scoped 主链足以先承接最小 review / escalation 语义

## 3. 这如何改变世界

制造现场很多交接失败，不是因为接班人没看到问题，而是因为问题被指出之后，系统仍然没有一个明确、可回放的升级动作。

只要没有显式 escalation action，平台就更像“会记录交接争议的系统”，而不是“会推动交接治理闭环的系统”。

## 4. 对自己的要求

- 不把 escalation action 夸大成审批结果
- 不把接收方 exception note 误写成发送方已经完成处置
- 不为了这一步引入过重的新状态机、projection 或 queue

## 5. 已经验证的事实

- `shift handoff` receipt 现在能通过显式 action 从 `acknowledged_with_exceptions` 进入 `escalated`
- 不处于 `acknowledged_with_exceptions` 的 receipt 会被拒绝
- 不匹配发送侧 accountable 角色的 actor 会被拒绝
- sandbox-safe file mode 重启后仍可回读 escalated receipt 与对应 `handoff_receipt_escalated` audit event

## 6. 这次做对了什么

这次做对的地方，是没有急着上 backlog、超时规则或 escalation engine，而是继续沿 task-scoped 主链把“异常交接包如何进入升级处理”这条最小路径补齐。

这样后续不论是扩 follow-up owner acceptance，还是扩 receipt overdue queue，都能沿同一条低风险、可验证的路线继续推进。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `shift handoff` 从“接收方能确认有问题”进入了“发送侧必须显式升级处理”。

这会直接影响后续路线：

- `handoff_receipt` 第一次具备完整的 `published -> acknowledged_with_exceptions -> escalated` 审计回放语义
- 交接闭环开始真正区别于 approval 链和 follow-up owner acceptance
- 后续 overdue / cross-shift queue 可以建立在已存在的显式治理动作之上，而不是建立在猜测上
