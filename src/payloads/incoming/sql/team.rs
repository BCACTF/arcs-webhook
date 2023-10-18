use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "query_name", rename_all = "snake_case")]
pub enum TeamQuery {
    #[serde(rename = "available")]
    CheckTeamnameAvailability {
        name: String,
    },
    #[serde(rename = "create")]
    CreateNewTeam {
        name: String,
        description: String,
        eligible: bool,
        affiliation: Option<String>,
        password: String,
    },
    #[serde(rename = "update")]
    UpdateTeam {
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        eligible: Option<bool>,
        affiliation: Option<Option<String>>,
        password: String,
    },
    #[serde(rename = "get")]
    GetTeam {
        id: Uuid,
    },
    #[serde(rename = "get_all")]
    GetAllTeams,
    
    #[serde(rename = "get_top")]
    GetTopTeams {
        limit: u32,
    },
    #[serde(rename = "get_top_history")]
    GetTopTeamsScoreHistory {
        limit: u32,
        start_time: NaiveDateTime,
    },
}
