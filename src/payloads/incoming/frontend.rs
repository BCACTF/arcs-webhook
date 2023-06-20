use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    Chall(Uuid),
    User(Uuid),
    Team(Uuid),
    Solves,
    All(),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToFrontend {
    Sync(SyncType),
}
