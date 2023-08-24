use crate::sql::CiText;
use serde::Serialize;
use uuid::Uuid;


#[derive(Debug, Clone, Serialize)]
pub struct SerializableTeam {
    pub id: Uuid,
    pub name: CiText,
    pub last_solve: Option<u64>,
    pub last_tiebreaker_solve: Option<u64>,
    pub eligible: bool,
    pub affiliation: Option<String>,
}
impl From<Team> for SerializableTeam {
    fn from(Team {
        id, name,
        last_solve, last_tiebreaker_solve,
        eligible, affiliation,
    }: Team) -> Self {
        SerializableTeam {
            id, name, eligible, affiliation,
            last_solve: last_solve.map(|dt| dt.timestamp() as u64),
            last_tiebreaker_solve: last_tiebreaker_solve.map(|dt| dt.timestamp() as u64),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(into = "SerializableTeam")]
pub struct Team {
    pub id: Uuid,
    pub name: CiText,
    pub last_solve: Option<chrono::NaiveDateTime>,
    pub last_tiebreaker_solve: Option<chrono::NaiveDateTime>,
    pub eligible: bool,
    pub affiliation: Option<String>,
}