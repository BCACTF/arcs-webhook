use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SerializableSolve {
    pub id: Uuid,
    
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub chall_id: Uuid,

    pub correct: bool,
    pub counted: bool,
    pub time: u64,
}
impl From<Solve> for SerializableSolve {
    fn from(Solve { id, user_id, team_id, chall_id, correct, counted, time }: Solve) -> Self {
        Self {
            id, user_id, team_id, chall_id, correct, counted,
            time: time.and_utc().timestamp() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(into = "SerializableSolve")]
pub struct Solve {
    pub id: Uuid,

    pub user_id: Uuid,
    pub team_id: Uuid,
    pub chall_id: Uuid,

    pub correct: bool,
    pub counted: bool,
    pub time: chrono::NaiveDateTime,
}


impl schemars::JsonSchema for Solve {
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        SerializableSolve::json_schema(gen)
    }
    fn schema_name() -> String {
        "Solve".to_string()
    }
}
