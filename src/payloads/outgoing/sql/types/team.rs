use crate::sql::CiText;
use serde::Serialize;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub struct SerializableTeam {
    pub id: Uuid,
    pub name: CiText,
    pub score: i32,
    pub last_solve: Option<u64>,
    pub eligible: bool,
    pub affiliation: Option<String>,
}
impl From<Team> for SerializableTeam {
    fn from(Team { id, name, score, last_solve, eligible, affiliation }: Team) -> Self {
        SerializableTeam {
            id, name, eligible, affiliation,
            score,
            last_solve: last_solve.map(|dt| dt.timestamp() as u64),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(into = "SerializableTeam")]
pub struct Team {
    pub id: Uuid,
    pub name: CiText,
    pub score: i32,
    pub last_solve: Option<chrono::NaiveDateTime>,
    pub eligible: bool,
    pub affiliation: Option<String>,
}


impl schemars::JsonSchema for Team {
    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        SerializableTeam::json_schema(gen)
    }
    fn schema_name() -> String {
        "Team".to_string()
    }
}


#[derive(Debug, Clone, Copy, Serialize, schemars::JsonSchema)]
pub struct ScoreEntry {
    pub team_id: Uuid,
    pub score: i32,
    pub time: chrono::NaiveDateTime,
}
