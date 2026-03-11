# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog, and this project uses Semantic Versioning.

## [Unreleased]

### Added

- Task and approval lifecycle models with guarded state transitions
- Initial connector read-only trait and audit sink abstractions
- M1 execution plan and lifecycle design baseline documents

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
