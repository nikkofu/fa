# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog, and this project uses Semantic Versioning.

## [Unreleased]

### Added

- Browser-based `FA Experience Command Center` at `/` for live monitoring, queue review, task dossier inspection, and quick workflow launches
- Aggregated UI overview endpoint at `GET /api/v1/experience/overview` for platform pulse, monitoring views, and queue previews
- First experience-layer workflow actions in the UI for approval, execution, completion, follow-up owner acceptance, and shift handoff acknowledgement / escalation
- Experience direction document covering enterprise UI/UX principles, IA, and phased roadmap

## [0.2.0] - 2026-03-13

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
- Filtered audit queries and task-scoped audit replay endpoints
- SQLite-backed task repository and audit store with `FA_SQLITE_DB_PATH` runtime injection
- Pilot workflow candidate evaluation and first workflow specification baseline documents
- Structured task evidence snapshot baseline and task-scoped evidence endpoint for pilot workflow alignment
- v0.2.0 test checklist, release checklist, and reusable workflow smoke script baseline
- Workflow responsibility matrix and approval strategy baseline with task-scoped governance endpoint
- Required approval-role enforcement for governed task approvals
- Sandbox-safe smoke validation path for restricted environments
- Manufacturing Agentic AI scenario landscape planning baseline
- Quality deviation candidate workflow specification baseline
- Quality workflow alignment checklist baseline
- High-frequency Agentic function priority map baseline
- Shift handoff workflow specification baseline
- Alert triage workflow specification baseline
- Follow-up and SLA model alignment checklist baseline
- Follow-up and SLA read model and query direction note baseline
- Follow-up task read model implementation cut note
- Follow-up task detail schema cut with backward-compatible `follow_up_items / follow_up_summary`
- Seeded `follow_up_item` draft for `shift handoff` requests
- Seeded `follow_up_item` draft for `alert triage` requests
- Explicit `follow-up-items/{follow_up_id}/accept-owner` action for task-scoped follow-up items
- Cross-task `GET /api/v1/follow-up-items` queue endpoint for aggregated follow-up owner reads
- Operational triage filters for `GET /api/v1/follow-up-items` covering blocked, escalation, due-before, risk, and priority
- Cross-task `GET /api/v1/follow-up-monitoring` endpoint for aggregated follow-up SLA monitoring reads
- Cross-shift `GET /api/v1/handoff-receipts` queue endpoint for aggregated receipt backlog reads
- Cross-shift `GET /api/v1/handoff-receipt-monitoring` endpoint for aggregated receipt monitoring reads
- Cross-task `GET /api/v1/alert-clusters` queue endpoint for aggregated alert-cluster backlog reads
- Cross-task `GET /api/v1/alert-cluster-monitoring` endpoint for aggregated alert-cluster monitoring reads
- Linked follow-up summary on `GET /api/v1/alert-clusters` queue items for owner and SLA visibility
- Linked follow-up triage filters on `GET /api/v1/alert-clusters` and `GET /api/v1/alert-cluster-monitoring`
- Linked follow-up monitoring aggregates on `GET /api/v1/alert-cluster-monitoring`
- Shift handoff workflow alignment checklist baseline
- Shift handoff receipt and acknowledgement direction note baseline
- Shift handoff receipt task read model implementation cut note
- Shift handoff receipt task detail schema cut with backward-compatible `handoff_receipt / handoff_receipt_summary`
- Seeded `handoff_receipt` draft for `shift handoff` requests
- Explicit `handoff-receipt/acknowledge` action for `shift handoff` receipts
- Explicit `handoff-receipt/escalate` action for `shift handoff` receipts with exceptions
- Alert triage workflow alignment checklist baseline
- Alert triage alert-cluster and event-ingestion direction note baseline
- Alert triage task read model implementation cut note
- Alert triage task detail schema cut with backward-compatible `alert_cluster_drafts / alert_triage_summary`
- Seeded `alert_cluster_draft` for `alert triage` requests
- Richer alert cluster source, line, triage-label, and window inference for `alert triage` requests
- Second `sustained_threshold_review` alert cluster draft mode for `scada` threshold-style triage requests
- Quality qms mock connector baseline note

### Changed

- Default local API port changed from `8080` to `8000`
- Added local development port convention to avoid conflicts with other projects
- Runtime persistence selection now prefers SQLite, then file-backed storage, then in-memory storage
- High-risk approval examples and smoke fixtures now use the governance-required approver role
- Default smoke data directories now live under the project-local `sandbox/`
- Task detail responses now include empty-by-default `follow_up_items / follow_up_summary`
- `shift handoff` requests now return a seeded `follow_up_item` draft and non-zero follow-up summary
- `alert triage` requests now return a seeded `follow_up_item` draft and non-zero follow-up summary
- Seeded task-scoped `follow_up_item` objects can now transition from `draft` to `accepted`
- Task repositories now expose cross-task listing to back `GET /api/v1/follow-up-items` in memory, file, and SQLite modes
- Follow-up queue reads now support blocked, escalation-required, due-before, risk, and priority filtering
- Follow-up monitoring reads now reuse follow-up queue filters to summarize open backlog ownership, urgency, and SLA buckets
- Task repositories now also back `GET /api/v1/handoff-receipts` for cross-shift receipt queue reads
- Task detail responses now include empty-by-default `handoff_receipt / handoff_receipt_summary`
- `shift handoff` requests now return a seeded `handoff_receipt` draft and non-zero receipt summary
- `shift handoff` receipts can now transition from `published` to `acknowledged / acknowledged_with_exceptions / escalated`
- Handoff receipt queue reads now support shift, receiving-role, receiving-actor, overdue, exception, and escalated filtering
- Handoff receipt monitoring reads now reuse receipt queue filters to summarize acknowledgement backlog, exceptions, escalation, and ack-window buckets
- Task detail responses now include empty-by-default `alert_cluster_drafts / alert_triage_summary`
- `alert triage` requests now return a seeded `alert_cluster_draft` and non-zero alert triage summary
- `alert triage` cluster drafts now infer `line_id` from request text or equipment ids and route sustained-threshold reviews to `maintenance_engineer`
- Alert cluster queue reads now support cluster-status, source-system, equipment, line, severity, triage-label, escalation, and window-overlap filtering
- Alert cluster queue reads now surface linked follow-up counts, acceptance, owner ids, and the highest-priority effective SLA state via explicit cluster refs or single-cluster alert-triage fallback
- Alert cluster queue reads now also support filtering by accepted follow-up owner, remaining unaccepted follow-up work, and follow-up escalation-required state
- Alert cluster monitoring reads now reuse alert-cluster queue filters, including linked follow-up triage dimensions, to summarize cluster backlog, severity, escalation, and window-state buckets
- Alert cluster monitoring reads now also summarize linked follow-up coverage, accepted/unaccepted backlog, escalation-required backlog, and worst-SLA buckets
- Sandbox smoke now also covers seeded alert-triage follow-up and alert-cluster draft persistence
- Sandbox smoke now also covers richer `scada` alert-cluster inference persistence after file-backed restart
- Sandbox smoke now also covers cross-task alert-cluster queue persistence and filtering after file-backed restart
- Sandbox smoke now also covers linked alert-cluster follow-up visibility after file-backed restart
- Sandbox smoke now also covers alert-cluster monitoring persistence and filtering after file-backed restart
- Sandbox smoke now also covers linked alert-cluster triage filters after file-backed restart
- Sandbox smoke now also covers linked alert-cluster monitoring aggregates after file-backed restart
- Sandbox smoke now also covers explicit follow-up owner acceptance persistence
- Sandbox smoke now also covers cross-task follow-up queue persistence and filtering
- Sandbox smoke now also covers queue triage filters after file-backed restart
- Sandbox smoke now also covers follow-up SLA monitoring persistence and filtering after file-backed restart
- Sandbox smoke now also covers explicit shift-handoff receipt acknowledgement persistence
- Sandbox smoke now also covers explicit shift-handoff receipt escalation persistence
- Sandbox smoke now also covers cross-shift handoff receipt queue persistence and filtering
- Sandbox smoke now also covers handoff receipt monitoring persistence and filtering after file-backed restart
- Rust workspace with `fa-domain`, `fa-core`, and `fa-server`
- Manufacturing-oriented domain model for enterprise, sites, workers, equipment, and task planning
- Initial orchestration engine that selects agentic patterns based on risk, scope, and approval needs
- HTTP endpoints for health, blueprint inspection, and task planning
- Project architecture, ADR, roadmap, release, and PM baseline documents
- GitHub Actions workflows for CI and tagged releases

## [0.1.0] - 2026-03-11

### Added

- Initial repository bootstrap and first executable platform skeleton
