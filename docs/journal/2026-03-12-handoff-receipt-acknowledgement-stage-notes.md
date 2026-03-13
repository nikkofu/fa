# Handoff Receipt Acknowledgement Stage Notes - 2026-03-12

## 1. 为什么这一层必须单独拿出来

只把 `handoff_receipt` 做成非空 draft 还不够，因为那只能证明平台看见了“有一份交接包”，不能证明平台真的知道“这份交接包已被接住”。

这一层单独拿出来，就是为了让 receipt 第一次拥有显式 acknowledgement action。

## 2. 这一阶段的创新点

这一阶段的关键，是没有把交接回执硬塞进审批接口，也没有一上来做 projection 或 queue，而是先加一条最小显式 action：

- 只对 `shift handoff` receipt 生效
- 只允许 `published -> acknowledged / acknowledged_with_exceptions`
- 只允许匹配 `receiving_role` 的 actor 执行

这意味着平台开始明确：

- acknowledgement 是独立于 approval 的协同动作
- role guard 可以先在现有任务 API 主链内成立
- receipt 闭环不需要等跨班次 backlog 才能出现

## 3. 这如何改变世界

制造现场很多交接失败，不是因为没人写摘要，而是因为系统根本不能表达“接班人有没有正式接住”。

只要没有显式 acknowledgement action，平台就更像“会生成交接包的系统”，而不是“会管理交接闭环的系统”。

## 4. 对自己的要求

- 不把 receipt acknowledged 夸大成 follow-up 已被接手
- 不把 acknowledgement action 错配成审批动作
- 不为了这一步引入过重的新状态机、projection 或 queue

## 5. 已经验证的事实

- `shift handoff` receipt 现在能通过显式 action 进入 `acknowledged`
- 提供 `exception_note` 时，receipt 能进入 `acknowledged_with_exceptions`
- 不匹配 `receiving_role` 的 actor 会被拒绝
- sandbox-safe file mode 重启后仍能回读 acknowledged receipt 和对应 audit event

## 6. 这次做对了什么

这次做对的地方，是没有急着去做 receipt backlog 或 escalation engine，而是先把显式 acknowledgement action 放进最稳定的 task-scoped 主链里。

这样后续不论是扩 review / escalation，还是扩 follow-up owner acceptance，都能沿同一条低风险、可验证的路线继续推进。

## 7. 这一步如何真正产生影响

这份代码阶段的真正价值，在于它让 `shift handoff` 从“有 receipt draft”进入了“有 receipt acknowledgement”。

这会直接影响后续路线：

- `acknowledged_with_exceptions` 有了清晰的 review / escalation 落点
- task audit 主链第一次能回放 handoff publish / acknowledge 语义
- 交接闭环开始真正区别于审批链和 follow-up owner acceptance
