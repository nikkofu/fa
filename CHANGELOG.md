# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog, and this project uses Semantic Versioning.

## [Unreleased]

### Added

- Task and approval lifecycle models with guarded state transitions
- Initial connector read-only trait and audit sink abstractions
- M1 execution plan and lifecycle design baseline documents
- Mock `MES` and mock `CMMS` read-only connector implementations
- In-memory audit sink and audit events endpoint
- Correlation-id aware intake flow with connector context hydration
- In-memory task store with `get/approve/execute` lifecycle endpoints
- Task lifecycle `complete / fail` actions
- Service-level integration tests covering the lifecycle happy path and failure path
- `TaskRepository` abstraction with injectable in-memory repository implementation
- Rejected task `resubmit` action with revision-loop service coverage
- File-backed task repository and audit store with `FA_DATA_DIR` runtime injection

### Changed

- Default local API port changed from `8080` to `8000`
- Added local development port convention to avoid conflicts with other projects
- Rust workspace with `fa-domain`, `fa-core`, and `fa-server`
- Manufacturing-oriented domain model for enterprise, sites, workers, equipment, and task planning
- Initial orchestration engine that selects agentic patterns based on risk, scope, and approval needs
- HTTP endpoints for health, blueprint inspection, and task planning
- Project architecture, ADR, roadmap, release, and PM baseline documents
- GitHub Actions workflows for CI and tagged releases

## [0.1.0] - 2026-03-11

### Added

- Initial repository bootstrap and first executable platform skeleton
