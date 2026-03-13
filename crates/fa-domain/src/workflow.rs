use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::patterns::AgenticPattern;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRequest {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub risk: TaskRisk,
    pub initiator: ActorHandle,
    pub stakeholders: Vec<ActorHandle>,
    pub equipment_ids: Vec<String>,
    pub integrations: Vec<IntegrationTarget>,
    pub desired_outcome: String,
    pub requires_human_approval: bool,
    pub requires_diagnostic_loop: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorHandle {
    pub id: String,
    pub display_name: String,
    pub role: String,
}

impl ActorHandle {
    pub fn normalized_role(&self) -> String {
        normalize_role_label(&self.role)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Routine,
    Expedited,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskRisk {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationTarget {
    Mes,
    Erp,
    Cmms,
    Quality,
    Scada,
    Warehouse,
    Safety,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub request_id: Uuid,
    pub patterns: Vec<AgenticPattern>,
    pub rationale: Vec<String>,
    pub approval_policy: ApprovalPolicy,
    #[serde(default)]
    pub governance: WorkflowGovernance,
    pub steps: Vec<PlannedStep>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalPolicy {
    Auto,
    OperationsSupervisor,
    SafetyOfficer,
    PlantManager,
}

impl ApprovalPolicy {
    pub fn requires_human_approval(self) -> bool {
        !matches!(self, ApprovalPolicy::Auto)
    }

    pub fn required_role(self) -> &'static str {
        match self {
            ApprovalPolicy::Auto => "system",
            ApprovalPolicy::OperationsSupervisor => "operations_supervisor",
            ApprovalPolicy::SafetyOfficer => "safety_officer",
            ApprovalPolicy::PlantManager => "plant_manager",
        }
    }

    pub fn escalation_role(self) -> Option<&'static str> {
        match self {
            ApprovalPolicy::Auto => None,
            ApprovalPolicy::OperationsSupervisor | ApprovalPolicy::SafetyOfficer => {
                Some("plant_manager")
            }
            ApprovalPolicy::PlantManager => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkflowGovernance {
    #[serde(default)]
    pub responsibility_matrix: Vec<ResponsibilityAssignment>,
    #[serde(default)]
    pub approval_strategy: ApprovalStrategy,
    #[serde(default)]
    pub fallback_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponsibilityAssignment {
    pub role: String,
    pub participation: GovernanceParticipation,
    pub responsibilities: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceParticipation {
    Responsible,
    Accountable,
    Consulted,
    Informed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalStrategy {
    pub policy: ApprovalPolicy,
    pub manual_approval_required: bool,
    pub required_role: String,
    pub escalation_role: Option<String>,
    pub decision_scope: Vec<String>,
    pub rationale: String,
}

impl Default for ApprovalStrategy {
    fn default() -> Self {
        Self {
            policy: ApprovalPolicy::Auto,
            manual_approval_required: false,
            required_role: ApprovalPolicy::Auto.required_role().to_string(),
            escalation_role: None,
            decision_scope: Vec::new(),
            rationale: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedStep {
    pub sequence: u8,
    pub label: String,
    pub owner: PlanOwner,
    pub expected_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanOwner {
    Human(String),
    Agent(String),
    System(String),
}

pub fn normalize_role_label(role: &str) -> String {
    let mut normalized = String::new();
    let mut last_was_separator = false;

    for ch in role.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator && !normalized.is_empty() {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    normalized.trim_matches('_').to_string()
}
