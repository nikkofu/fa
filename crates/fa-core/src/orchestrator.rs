use chrono::Utc;

use fa_domain::{
    AgenticPattern, ApprovalPolicy, ExecutionPlan, PlanOwner, PlannedStep, TaskPriority,
    TaskRequest, TaskRisk,
};

use crate::blueprint::{bootstrap_blueprint, PlatformBlueprint};

#[derive(Debug, Clone)]
pub struct WorkOrchestrator {
    blueprint: PlatformBlueprint,
}

impl Default for WorkOrchestrator {
    fn default() -> Self {
        Self {
            blueprint: bootstrap_blueprint(),
        }
    }
}

impl WorkOrchestrator {
    pub fn new(blueprint: PlatformBlueprint) -> Self {
        Self { blueprint }
    }

    pub fn blueprint(&self) -> &PlatformBlueprint {
        &self.blueprint
    }

    pub fn plan_task(&self, request: TaskRequest) -> ExecutionPlan {
        let patterns = select_patterns(&request);
        let approval_policy = select_approval_policy(&request);
        let steps = build_steps(&request, &patterns, approval_policy);
        let rationale = build_rationale(&request, &patterns, approval_policy);

        ExecutionPlan {
            request_id: request.id,
            patterns,
            rationale,
            approval_policy,
            steps,
            created_at: Utc::now(),
        }
    }
}

fn select_patterns(request: &TaskRequest) -> Vec<AgenticPattern> {
    let mut patterns = Vec::new();

    if matches!(request.risk, TaskRisk::Low)
        && request.integrations.len() <= 1
        && !request.requires_human_approval
        && !request.requires_diagnostic_loop
    {
        patterns.push(AgenticPattern::SingleAgent);
    } else {
        patterns.push(AgenticPattern::Coordinator);
    }

    patterns.push(AgenticPattern::DeterministicWorkflow);

    if request.requires_diagnostic_loop
        || matches!(request.priority, TaskPriority::Critical)
        || request.description.to_ascii_lowercase().contains("diagnos")
    {
        patterns.push(AgenticPattern::ReActLoop);
    }

    if request.requires_human_approval
        || matches!(request.risk, TaskRisk::High | TaskRisk::Critical)
    {
        patterns.push(AgenticPattern::HumanInTheLoop);
    }

    if request.integrations.len() > 1 || !request.equipment_ids.is_empty() {
        patterns.push(AgenticPattern::CustomBusinessLogic);
    }

    patterns
}

fn select_approval_policy(request: &TaskRequest) -> ApprovalPolicy {
    if matches!(request.risk, TaskRisk::Critical) {
        ApprovalPolicy::PlantManager
    } else if request.requires_human_approval || matches!(request.risk, TaskRisk::High) {
        ApprovalPolicy::SafetyOfficer
    } else if matches!(request.priority, TaskPriority::Critical) {
        ApprovalPolicy::OperationsSupervisor
    } else {
        ApprovalPolicy::Auto
    }
}

fn build_steps(
    request: &TaskRequest,
    patterns: &[AgenticPattern],
    approval_policy: ApprovalPolicy,
) -> Vec<PlannedStep> {
    let mut steps = vec![PlannedStep {
        sequence: 1,
        label: "Intake and context hydration".to_string(),
        owner: PlanOwner::System("workflow-engine".to_string()),
        expected_output: format!(
            "Validated request, enterprise context, and connector read plan for '{}'",
            request.title
        ),
    }];

    if patterns.contains(&AgenticPattern::SingleAgent) {
        steps.push(PlannedStep {
            sequence: 2,
            label: "Single-agent planning".to_string(),
            owner: PlanOwner::Agent("ops-copilot".to_string()),
            expected_output: "A concise action plan with evidence and confidence".to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::Coordinator) {
        steps.push(PlannedStep {
            sequence: 2,
            label: "Coordinator task graph".to_string(),
            owner: PlanOwner::Agent("orchestrator".to_string()),
            expected_output: "Delegation map covering people, systems, and domain agents"
                .to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::ReActLoop) {
        steps.push(PlannedStep {
            sequence: 3,
            label: "Evidence-driven reasoning loop".to_string(),
            owner: PlanOwner::Agent("reasoning-loop".to_string()),
            expected_output: "Hypotheses, evidence citations, and next-best-action recommendations"
                .to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::CustomBusinessLogic) {
        steps.push(PlannedStep {
            sequence: 4,
            label: "Policy and connector orchestration".to_string(),
            owner: PlanOwner::System("policy-engine".to_string()),
            expected_output: "A deterministic connector sequence bounded by SOP and safety policy"
                .to_string(),
        });
    }

    if patterns.contains(&AgenticPattern::HumanInTheLoop) {
        steps.push(PlannedStep {
            sequence: 5,
            label: "Human approval checkpoint".to_string(),
            owner: PlanOwner::Human(format!("{approval_policy:?}")),
            expected_output: "Approved, rejected, or revised execution package".to_string(),
        });
    }

    steps.push(PlannedStep {
        sequence: 6,
        label: "Execution, audit, and close-out".to_string(),
        owner: PlanOwner::System("execution-runner".to_string()),
        expected_output: "Task outcome, audit trail, and follow-up actions".to_string(),
    });

    steps
}

fn build_rationale(
    request: &TaskRequest,
    patterns: &[AgenticPattern],
    approval_policy: ApprovalPolicy,
) -> Vec<String> {
    let mut rationale = vec![format!(
        "Task '{}' is classified as {:?} risk and {:?} priority.",
        request.title, request.risk, request.priority
    )];

    if patterns.contains(&AgenticPattern::Coordinator) {
        rationale.push(
            "Coordinator orchestration was selected because the task spans multiple actors, systems, or equipment."
                .to_string(),
        );
    }

    if patterns.contains(&AgenticPattern::SingleAgent) {
        rationale.push(
            "Single-agent handling is sufficient because the task is low-risk and operationally narrow."
                .to_string(),
        );
    }

    if patterns.contains(&AgenticPattern::ReActLoop) {
        rationale.push(
            "A ReAct-style loop is required to iterate over evidence before recommending action."
                .to_string(),
        );
    }

    if patterns.contains(&AgenticPattern::CustomBusinessLogic) {
        rationale.push(
            "Custom business logic is required to enforce deterministic SOP, connector sequencing, and safety boundaries."
                .to_string(),
        );
    }

    rationale.push(format!(
        "Approval policy is {:?}, aligning with governance for manufacturing change and safety.",
        approval_policy
    ));

    rationale
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use fa_domain::{ActorHandle, IntegrationTarget};

    use super::*;

    fn base_request() -> TaskRequest {
        TaskRequest {
            id: Uuid::new_v4(),
            title: "Investigate spindle temperature drift".to_string(),
            description: "Diagnose repeated spindle temperature drift before the next shift."
                .to_string(),
            priority: TaskPriority::Expedited,
            risk: TaskRisk::Medium,
            initiator: ActorHandle {
                id: "worker_1001".to_string(),
                display_name: "Liu Supervisor".to_string(),
                role: "Production Supervisor".to_string(),
            },
            stakeholders: Vec::new(),
            equipment_ids: vec!["eq_cnc_01".to_string()],
            integrations: vec![IntegrationTarget::Mes, IntegrationTarget::Cmms],
            desired_outcome: "Recover stable spindle temperature within tolerance".to_string(),
            requires_human_approval: false,
            requires_diagnostic_loop: true,
        }
    }

    #[test]
    fn simple_low_risk_work_uses_single_agent() {
        let orchestrator = WorkOrchestrator::default();
        let mut request = base_request();
        request.title = "Summarize shift notes".to_string();
        request.description = "Summarize shift notes for morning handoff.".to_string();
        request.priority = TaskPriority::Routine;
        request.risk = TaskRisk::Low;
        request.integrations = vec![IntegrationTarget::Mes];
        request.equipment_ids.clear();
        request.requires_diagnostic_loop = false;

        let plan = orchestrator.plan_task(request);

        assert!(plan.patterns.contains(&AgenticPattern::SingleAgent));
        assert!(!plan.patterns.contains(&AgenticPattern::HumanInTheLoop));
        assert_eq!(plan.approval_policy, ApprovalPolicy::Auto);
    }

    #[test]
    fn critical_work_requires_human_governance() {
        let orchestrator = WorkOrchestrator::default();
        let mut request = base_request();
        request.priority = TaskPriority::Critical;
        request.risk = TaskRisk::Critical;
        request.requires_human_approval = true;

        let plan = orchestrator.plan_task(request);

        assert!(plan.patterns.contains(&AgenticPattern::Coordinator));
        assert!(plan.patterns.contains(&AgenticPattern::ReActLoop));
        assert!(plan.patterns.contains(&AgenticPattern::HumanInTheLoop));
        assert!(plan.patterns.contains(&AgenticPattern::CustomBusinessLogic));
        assert_eq!(plan.approval_policy, ApprovalPolicy::PlantManager);
    }
}
