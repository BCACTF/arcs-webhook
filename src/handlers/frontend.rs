use async_trait::async_trait;
use serde_json::json;

use crate::logging::*;

use crate::http_client::DEFAULT;
use crate::payloads::incoming::frontend::SyncType;
use crate::payloads::incoming::ToFrontend;
use crate::payloads::outgoing::frontend::{FromFrontend, FromFrontendErr};

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToFrontend {
    type SuccessPayload = FromFrontend;
    type ErrorPayload = FromFrontendErr;
    async fn handle(self) -> ResponseFrom<Self> {
        let payload = match self {
            Self::Sync(sync_type) => match sync_type {
                SyncType::All       => json!({ "__type": "all" }),
                SyncType::Solves    => json!({ "__type": "solves" }),
                
                SyncType::AllChalls => json!({ "__type": "all_challs" }),
                SyncType::AllTeams  => json!({ "__type": "all_teams" }),
                SyncType::AllUsers  => json!({ "__type": "all_users" }),

                SyncType::Chall(id) => json!({
                    "__type": "chall",
                    "id": id,
                }),
                SyncType::Team(id) => json!({
                    "__type": "team",
                    "id": id,
                }),
                SyncType::User(id) => json!({
                    "__type": "user",
                    "id": id,
                }),
            }
        };

        let response = DEFAULT
            .post(format!("{}/api/sync", crate::env::frontend_address()))
            .bearer_auth(String::from_utf8_lossy(&crate::auth::webhook_auth()))
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(response) => if response.status().is_success() {
                info!("Frontend req success");
                
                match self {
                    Self::Sync(sync_type) => Ok(FromFrontend::Synced(sync_type)),
                }
            } else {
                if let Ok(bytes) = response.bytes().await {
                    debug!("{}", String::from_utf8_lossy(&bytes));
                }
                warn!("Frontend req returned error");
                
                match self {
                    Self::Sync(sync_type) => Err(FromFrontendErr::FailedToSync(sync_type)),
                }
            },
            Err(e) => {
                error!("Sending request to frontend failed. This could signal a major issue.");

                Err(FromFrontendErr::WebhookServerError(e.to_string()))
            }
        }
    }
}
