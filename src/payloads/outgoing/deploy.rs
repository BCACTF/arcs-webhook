use std::time::Duration;

use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::handlers::OutgoingErr;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]

#[serde(rename_all = "snake_case")]
pub enum Status {
    Started,
    Building,
    Pulling,
    Pushing,
    Uploading,
    Success,

    Failure,

    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeploymentStatus {
    pub (crate) status: Status,
    pub (crate) status_time: Duration,
    pub (crate) chall_name: Option<String>,
    pub (crate) poll_id: Uuid,

    pub (crate) err_msg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "__type", rename_all = "snake_case", content = "data")]
pub enum FromDeploy {
    Status(DeploymentStatus),
    ChallNameList(Vec<String>),
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub enum FromDeployErr {
    BadSend,
    BadResponse,
    DbError,
    DeployServer {
        code: u16,
        body: Vec<u8>,
    },
}

impl OutgoingErr for FromDeployErr {
    fn body(self) -> Result<serde_json::Value, String> {
        match self {
            Self::BadSend => Ok(serde_json::json!("Failed to forward request to the deploy server")),
            Self::BadResponse => Ok(serde_json::json!("The deploy server responded with an invalid data shape.")),
            Self::DbError => Ok(serde_json::json!("There was a database issue that prevented the deploy message from being sent.")),
            Self::DeployServer { body, .. } => Ok(serde_json::json!(String::from_utf8_lossy(&body)))
        }
    }
    fn status_code(&self) -> u16 {
        match self {
            Self::BadSend | Self::BadResponse | Self::DbError => 500,
            Self::DeployServer { code, .. } => *code
        }
    }
}
