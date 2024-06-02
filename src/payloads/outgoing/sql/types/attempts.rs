use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SerializableAttempts {
    pub chall_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub correct: u64,
    pub incorrect: u64,
}
impl From<Attempts> for SerializableAttempts {
    fn from(Attempts { chall_id, team_id, correct, incorrect }: Attempts) -> Self {
        Self {
            chall_id, team_id,
            correct, incorrect
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(into = "SerializableAttempts")]
pub struct Attempts {
    pub chall_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub correct: u64,
    pub incorrect: u64,
}

impl schemars::JsonSchema for Attempts {
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        SerializableAttempts::json_schema(gen)
    }
    fn schema_name() -> String {
        "Attempts".to_string()
    }
}