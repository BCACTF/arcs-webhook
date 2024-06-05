use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__query_name", rename_all = "snake_case", content = "params")]
pub enum HistoryQuery {
    #[serde(rename = "get_team_histories")]
    GetTeamHistories { team_ids: Vec<Uuid> },
    #[serde(rename = "get_top_team_histories")]
    GetTopTeamHistories { limit: u32 },
}
