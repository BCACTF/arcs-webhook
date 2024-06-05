use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SimpleHistoryEntry {
    pub points: u64,
    pub time: u64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct History {
    pub team_id: Uuid,
    pub entries: Vec<SimpleHistoryEntry>,
}
