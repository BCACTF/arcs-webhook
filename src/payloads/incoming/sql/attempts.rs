use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__query_name", rename_all = "snake_case", content = "params")]
pub enum AttemptQuery {
    #[serde(rename = "get_all_attempts_by_chall")]
    GetAllAttemptsByChall { chall_id: Uuid },
    #[serde(rename = "get_all_attempts_by_team")]
    GetAllAttemptsByTeam { team_id: Uuid },
    #[serde(rename = "get_all_attempts")]
    GetAllAttempts,
}
