use async_trait::async_trait;

use crate::logging::*;
use crate::http_client::DEFAULT;

use crate::payloads::incoming::ToDeploy;
use crate::payloads::incoming::deploy::ChallIdentifier;

use crate::payloads::outgoing::deploy::{FromDeploy, FromDeployErr};

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToDeploy {
    type SuccessPayload = FromDeploy;
    type ErrorPayload = FromDeployErr;
    async fn handle(self) -> ResponseFrom<Self> {
        trace!("Handling deploy req");

        let (req_type, polling_id, chall_name) = match self {
            Self::Deploy { chall } => (
                "deploy",
                uuid::Uuid::new_v4(),
                match chall {
                    ChallIdentifier::CurrDeployedId(id) => id.to_string(),
                    ChallIdentifier::Folder(s) => s,
                }
            ),
            Self::Poll { id } => ("poll", id, "aaaaaaaaa".to_string()),
            Self::Remove { chall } => ("delete", chall, "jxcz".to_string())
        };


        // FIXME: Make the deploy server not have a different number of underscores
        let body = serde_json::json!({
            "_type": req_type,
            "deploy_identifier": polling_id,
            "chall_name": chall_name,
        });

        let response = DEFAULT
            .post(crate::env::deploy_address())
            .bearer_auth(crate::env::webhook_token())
            .json(&body)
            .send()
            .await;


        match response {
            Ok(response) => if response.status().is_success() {
                let data = match response.json().await {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Bad response shape from deploy server: {e}");
                        return Err(FromDeployErr::BadResponse);
                    }
                };

                info!("Deploy req successful");
                Ok(FromDeploy::Status(data))
            } else {
                warn!("Deploy req failed");
                debug!("Response data: {response:#?}");

                let code = response.status().as_u16();

                let err = match response.bytes().await {
                    Ok(body) => FromDeployErr::DeployServer {
                        code,
                        body: body.to_vec(),
                    },
                    Err(_) => FromDeployErr::DeployServer {
                        code: 500,
                        body: "Failed to read deploy server response".into(),
                    }
                };
                Err(err)
            },
            Err(_) => {
                error!("Sending request to the deploy server failed. This could signal a major issue.");
                Err(FromDeployErr::BadSend)
            },
        }
    }
}