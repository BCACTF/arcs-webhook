use async_trait::async_trait;
use uuid::Uuid;

use crate::logging::*;
use crate::http_client::DEFAULT;

use crate::payloads::incoming::ToDeploy;
use crate::payloads::incoming::deploy::ChallIdentifier;

use crate::payloads::outgoing::deploy::{FromDeploy, FromDeployErr};

use super::sql::{ get_chall_id_by_source_folder, get_chall_source_folder_by_id };

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToDeploy {
    type SuccessPayload = FromDeploy;
    type ErrorPayload = FromDeployErr;
    async fn handle(self) -> ResponseFrom<Self> {
        trace!("Handling deploy req");

        #[derive(Debug, serde::Serialize)]
        struct Modifications {
            name: Option<String>,
            desc: Option<String>,
            points: Option<u64>,
            categories: Option<Vec<String>>,
            tags: Option<Option<Vec<String>>>,
        }

        let (req_type, polling_id, chall_name, modifications) = match self {
            Self::Deploy { chall, force_wipe } => {
                let id = if force_wipe { uuid::Uuid::new_v4() } else {
                    match &chall {
                        &ChallIdentifier::CurrDeployedId(id) => id,
                        ChallIdentifier::Folder(source_folder) => {
                            let id = match get_chall_id_by_source_folder(source_folder).await {
                                Ok(id) => id,
                                Err(e) => {
                                    debug!("Database error: {e}");
                                    return Err(FromDeployErr::DbError);
                                }

                            };
                            id.unwrap_or_else(uuid::Uuid::new_v4)
                        }
                    }
                };
                ( // FIXME: If force_wipe isn't enabled, try to access the current ID.
                    "deploy",
                    id,
                    match chall {
                        ChallIdentifier::CurrDeployedId(id) => id.to_string(),
                        ChallIdentifier::Folder(s) => s,
                    },
                    None
                )
            },
            Self::Poll { id } => ("poll", id, "".to_string(), None),
            Self::Remove { chall } => ("delete", chall, "".to_string(), None),
            Self::ModifyMeta {
                id,
                name, desc, points, categories, tags,
            } => {
                let chall_source_folder = match get_chall_source_folder_by_id(id).await {
                    Ok(Some(id)) => id,
                    Ok(None) => return Err(FromDeployErr::DbError),
                    Err(e) => {
                        debug!("Database error: {e}");
                        return Err(FromDeployErr::DbError);
                    }
                };

                (
                    "modify_meta",
                    id,
                    chall_source_folder,
                    Some(Modifications { name, desc, points, categories, tags }),
                )
            },
        };


        // FIXME: Make the deploy server not have a different number of underscores
        let body = serde_json::json!({
            "__type": req_type,
            "deploy_identifier": polling_id,
            "chall_name": chall_name,
            "modifications": modifications,
        });

        let response = DEFAULT
            .post(crate::env::deploy_address())
            .bearer_auth(String::from_utf8_lossy(&crate::auth::webhook_auth()))
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