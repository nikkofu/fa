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
