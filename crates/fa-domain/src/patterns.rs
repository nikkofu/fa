use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgenticPattern {
    SingleAgent,
    Coordinator,
    ReActLoop,
    HumanInTheLoop,
    DeterministicWorkflow,
    CustomBusinessLogic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    Reasoning,
    Orchestration,
    Governance,
    Integration,
}
