use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query_name", rename_all = "snake_case")]
pub enum SolveQuery {
    #[serde(rename = "get_all")]
    GetAllSolves,

    #[serde(rename = "get")]
    GetSolve { id: Uuid },
    
    #[serde(rename = "get_chall")]
    GetAllSolvesByChall { chall_id: Uuid },
    #[serde(rename = "get_team")]
    GetAllSolvesByTeam { team_id: Uuid },
    #[serde(rename = "get_user")]
    GetAllSolvesByUser { user_id: Uuid },

    #[serde(rename = "attempt")]
    AttemptSolve {
        user_id: Uuid, team_id: Uuid, chall_id: Uuid,
        user_auth: super::Auth,
        flag_guess: String,
    },
    
}
