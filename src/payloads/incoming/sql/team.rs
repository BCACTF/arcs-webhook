use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    GetAllTeams
}