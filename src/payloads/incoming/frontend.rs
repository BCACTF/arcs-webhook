use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__sync_type", rename_all = "snake_case", content = "id")]
pub enum SyncType {
    Chall(Uuid),
    AllChalls,
    User(Uuid),
    AllUsers,
    Team(Uuid),
    AllTeams,
    Solves,
    All,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToFrontend {
    Sync(SyncType),
}
