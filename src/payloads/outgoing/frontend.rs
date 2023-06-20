use reqwest::StatusCode;
use serde::Serialize;
use uuid::Uuid;

use crate::handlers::OutgoingErr;

#[derive(Debug, Clone, Serialize)]
pub enum SyncType {
    Chall(Uuid),
    User(Uuid),
    Team(Uuid),
    Solves,
    All,
}

#[derive(Debug, Clone, Serialize)]
pub enum FromFrontend {
    Synced(SyncType),
}

#[derive(Debug, Clone, Serialize)]
pub enum FromFrontendErr {
    FailedToSync(SyncType),
}

impl OutgoingErr for FromFrontendErr {
    fn status_code(&self) -> u16 {
        StatusCode::INTERNAL_SERVER_ERROR.as_u16()
    }
    fn body(self) -> Result<serde_json::Value, String> {
        use serde_json::json;
        
        match self {
            Self::FailedToSync(sync_type) => {
                let (sync_type, id) = match sync_type {
                    SyncType::User(id) => ("user", Some(id)),
                    SyncType::Team(id) => ("team", Some(id)),
                    SyncType::Chall(id) => ("chall", Some(id)),
                    SyncType::Solves => ("solves", None),
                    SyncType::All => ("all", None),
                };
                let sync_type = if let Some(id) = id {
                    json!({
                        "type": sync_type,
                        "id": id
                    })
                } else {
                    json!({
                        "type": sync_type
                    })
                };

                Ok(json!({
                    "message": "failed to sync",
                    "sync_type": sync_type
                }))
            }
        }
    }
}
