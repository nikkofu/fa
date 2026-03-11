use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnterpriseContext {
    pub enterprise_name: String,
    pub organizations: Vec<Organization>,
    pub sites: Vec<OperatingSite>,
    pub lines: Vec<ManufacturingLine>,
    pub workers: Vec<Worker>,
    pub agents: Vec<AgentProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatingSite {
    pub id: String,
    pub name: String,
    pub country_code: String,
    pub time_zone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManufacturingLine {
    pub id: String,
    pub site_id: String,
    pub name: String,
    pub criticality: String,
    pub equipment: Vec<Equipment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub class: EquipmentClass,
    pub protocol: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipmentClass {
    Cnc,
    Plc,
    Robot,
    Sensor,
    Vision,
    Packaging,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Worker {
    pub id: String,
    pub name: String,
    pub role: String,
    pub organization_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub remit: String,
    pub supervised_by_role: String,
}
