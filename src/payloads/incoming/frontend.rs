use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "__sync_type", rename_all = "snake_case")]
pub enum SyncType {
    Chall(Uuid),
    User(Uuid),
    Team(Uuid),
    Solves,
    All,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToFrontend {
    Sync(SyncType),
}
