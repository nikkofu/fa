# Release Process

## 1. 版本策略

项目采用 Semantic Versioning：

- `MAJOR`: 不兼容接口或架构变更
- `MINOR`: 向后兼容的新能力
- `PATCH`: 缺陷修复和非破坏性改进

## 2. 分支与发布约束

- `main` 保持可构建
- 所有版本说明同步到 `CHANGELOG.md`
- GitHub tag 使用 `vX.Y.Z`

## 3. 发布前检查

发布前必须完成：

1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `bash scripts/smoke_v0_2_0.sh`
5. `bash scripts/smoke_v0_2_0_sandbox.sh`
6. `git diff --check`
7. 更新 `CHANGELOG.md`、`README.md` 与版本 release note
8. 确认试运行或目标环境的回退方案与人工接管策略

说明：

- 若当前执行环境不允许本地监听 TCP 端口，至少先运行 `bash scripts/smoke_v0_2_0_sandbox.sh` 做受限环境验证
- 正式 release gate 仍以真实 listener 的 `scripts/smoke_v0_2_0.sh` 为准
- 对外发布说明统一归档到 `docs/release/`，当前版本为 `docs/release/v0.2.0-release-notes.md`

参考清单：

- [docs/qa/v0.2.0-test-checklist.md](qa/v0.2.0-test-checklist.md)
- [docs/qa/v0.2.0-release-checklist.md](qa/v0.2.0-release-checklist.md)
- [docs/qa/v0.2.0-manual-validation.md](qa/v0.2.0-manual-validation.md)
- [docs/release/v0.2.0-release-notes.md](release/v0.2.0-release-notes.md)

## 4. 发布动作

1. 在 `main` 更新版本号与 release note
2. 提交 changelog、README、发布与验证文档
3. 打 tag，例如 `v0.2.0`
4. push `main` 与 tag 到 GitHub
5. 由 GitHub Actions 自动执行 release workflow 并创建 GitHub release
6. 检查 artifact、发布说明、已知风险和回退说明是否完整

## 5. Release note 模板

每个 release 至少包含：

- 本次新增能力
- 本次修复问题
- 影响范围
- 升级注意事项
- 已知风险

## 6. 试运行前额外要求

对涉及制造现场的版本，还必须补充：

- SOP 影响评估
- 审批责任人清单
- 回退与人工接管策略
- 数据保留和审计策略
