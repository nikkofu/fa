use serde::Serialize;

use fa_domain::{
    AgentProfile, AgenticPattern, EnterpriseContext, Equipment, EquipmentClass, ManufacturingLine,
    OperatingSite, Organization, PatternCategory, Worker,
};

#[derive(Debug, Clone, Serialize)]
pub struct PlatformBlueprint {
    pub platform_name: String,
    pub version: String,
    pub vision: String,
    pub selected_patterns: Vec<PatternDecision>,
    pub system_layers: Vec<SystemLayer>,
    pub delivery_tracks: Vec<DeliveryTrack>,
    pub reference_enterprise: EnterpriseContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct PatternDecision {
    pub pattern: AgenticPattern,
    pub category: PatternCategory,
    pub why_selected: String,
    pub guardrails: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemLayer {
    pub name: String,
    pub responsibilities: Vec<String>,
    pub initial_rust_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeliveryTrack {
    pub name: String,
    pub milestone: String,
    pub success_metric: String,
}

pub fn bootstrap_blueprint() -> PlatformBlueprint {
    PlatformBlueprint {
        platform_name: "FA Manufacturing Agentic Platform".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        vision: "Use AI agents to coordinate people, equipment, and enterprise systems with clear safety, audit, and human-approval boundaries."
            .to_string(),
        selected_patterns: vec![
            PatternDecision {
                pattern: AgenticPattern::Coordinator,
                category: PatternCategory::Orchestration,
                why_selected: "Manufacturing tasks usually cross ERP, MES, CMMS, quality, and shopfloor actors. A coordinator keeps routing and accountability explicit."
                    .to_string(),
                guardrails: "The coordinator never writes to physical equipment directly without policy evaluation and connector-level safety rules."
                    .to_string(),
            },
            PatternDecision {
                pattern: AgenticPattern::ReActLoop,
                category: PatternCategory::Reasoning,
                why_selected: "Fault diagnosis, order exceptions, and production planning require iterative reasoning across telemetry, work instructions, and business context."
                    .to_string(),
                guardrails: "Every reasoning loop must emit evidence, recommended action, and confidence before execution."
                    .to_string(),
            },
            PatternDecision {
                pattern: AgenticPattern::HumanInTheLoop,
                category: PatternCategory::Governance,
                why_selected: "Safety, quality, and cost-critical decisions need explicit approval by supervisors or responsible engineers."
                    .to_string(),
                guardrails: "High-risk and safety-adjacent workflows require approval and are fully audit logged."
                    .to_string(),
            },
            PatternDecision {
                pattern: AgenticPattern::CustomBusinessLogic,
                category: PatternCategory::Integration,
                why_selected: "Each factory has local SOPs, machine states, and release rules that cannot be delegated to generic LLM behavior."
                    .to_string(),
                guardrails: "Deterministic policy checks wrap all external system writes and equipment-affecting actions."
                    .to_string(),
            },
            PatternDecision {
                pattern: AgenticPattern::DeterministicWorkflow,
                category: PatternCategory::Orchestration,
                why_selected: "Routine production work, escalation, and close-out steps should remain predictable and measurable."
                    .to_string(),
                guardrails: "Standard work templates define the required evidence and exit conditions for each workflow."
                    .to_string(),
            },
        ],
        system_layers: vec![
            SystemLayer {
                name: "experience-and-api".to_string(),
                responsibilities: vec![
                    "Expose planning, approval, execution, and audit APIs.".to_string(),
                    "Serve operator, supervisor, and integrator-facing workflows.".to_string(),
                ],
                initial_rust_components: vec!["axum".to_string(), "serde".to_string()],
            },
            SystemLayer {
                name: "agent-orchestration".to_string(),
                responsibilities: vec![
                    "Route tasks to the right agentic pattern.".to_string(),
                    "Manage planning, approval gates, and execution state.".to_string(),
                ],
                initial_rust_components: vec![
                    "fa-core::WorkOrchestrator".to_string(),
                    "policy engine".to_string(),
                ],
            },
            SystemLayer {
                name: "domain-and-connectors".to_string(),
                responsibilities: vec![
                    "Model enterprise, people, devices, and workflow semantics.".to_string(),
                    "Bridge ERP, MES, CMMS, SCADA, and edge agents.".to_string(),
                ],
                initial_rust_components: vec![
                    "fa-domain".to_string(),
                    "connector adapters".to_string(),
                ],
            },
            SystemLayer {
                name: "observability-and-governance".to_string(),
                responsibilities: vec![
                    "Record approvals, evidence, and execution outcomes.".to_string(),
                    "Support traceability, change control, and release readiness.".to_string(),
                ],
                initial_rust_components: vec![
                    "audit log".to_string(),
                    "metrics and tracing".to_string(),
                ],
            },
        ],
        delivery_tracks: vec![
            DeliveryTrack {
                name: "factory-copilot-foundation".to_string(),
                milestone: "M1".to_string(),
                success_metric: "A shared domain model, planner API, and supervisor approvals are running in dev."
                    .to_string(),
            },
            DeliveryTrack {
                name: "business-system-integration".to_string(),
                milestone: "M2".to_string(),
                success_metric: "MES, ERP, and CMMS connectors support safe read flows and controlled write pilots."
                    .to_string(),
            },
            DeliveryTrack {
                name: "edge-execution-pilot".to_string(),
                milestone: "M3".to_string(),
                success_metric: "A pilot line can execute one closed-loop workflow with audit and human fallback."
                    .to_string(),
            },
        ],
        reference_enterprise: EnterpriseContext {
            enterprise_name: "Nikkofu Manufacturing".to_string(),
            organizations: vec![
                Organization {
                    id: "org_ops".to_string(),
                    name: "Operations".to_string(),
                    function: "Production execution and supervision".to_string(),
                },
                Organization {
                    id: "org_quality".to_string(),
                    name: "Quality".to_string(),
                    function: "Quality assurance and release".to_string(),
                },
                Organization {
                    id: "org_maintenance".to_string(),
                    name: "Maintenance".to_string(),
                    function: "Equipment reliability and repairs".to_string(),
                },
            ],
            sites: vec![OperatingSite {
                id: "site_sz".to_string(),
                name: "Shenzhen Plant".to_string(),
                country_code: "CN".to_string(),
                time_zone: "Asia/Shanghai".to_string(),
            }],
            lines: vec![ManufacturingLine {
                id: "line_01".to_string(),
                site_id: "site_sz".to_string(),
                name: "Precision Assembly Line 01".to_string(),
                criticality: "high".to_string(),
                equipment: vec![
                    Equipment {
                        id: "eq_cnc_01".to_string(),
                        name: "CNC-01".to_string(),
                        class: EquipmentClass::Cnc,
                        protocol: "OPC-UA".to_string(),
                    },
                    Equipment {
                        id: "eq_robot_01".to_string(),
                        name: "Robot Arm A".to_string(),
                        class: EquipmentClass::Robot,
                        protocol: "Profinet".to_string(),
                    },
                    Equipment {
                        id: "eq_vision_01".to_string(),
                        name: "Vision Station".to_string(),
                        class: EquipmentClass::Vision,
                        protocol: "REST".to_string(),
                    },
                ],
            }],
            workers: vec![
                Worker {
                    id: "worker_1001".to_string(),
                    name: "Liu Supervisor".to_string(),
                    role: "Production Supervisor".to_string(),
                    organization_id: "org_ops".to_string(),
                },
                Worker {
                    id: "worker_2001".to_string(),
                    name: "Chen QE".to_string(),
                    role: "Quality Engineer".to_string(),
                    organization_id: "org_quality".to_string(),
                },
            ],
            agents: vec![
                AgentProfile {
                    id: "agent_ops_copilot".to_string(),
                    name: "Ops Copilot".to_string(),
                    remit: "Production coordination and decision support".to_string(),
                    supervised_by_role: "Production Supervisor".to_string(),
                },
                AgentProfile {
                    id: "agent_maintenance".to_string(),
                    name: "Maintenance Analyst".to_string(),
                    remit: "Equipment diagnosis and maintenance planning".to_string(),
                    supervised_by_role: "Maintenance Lead".to_string(),
                },
            ],
        },
    }
}
