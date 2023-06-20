use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChallIdentifier {
    CurrDeployedId(Uuid), // NOTE: This is at the top for higher deserialization priority. DON'T MOVE IT PLEASE.
    Folder(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToDeploy {
    Deploy { chall: ChallIdentifier },
    Poll { id: Uuid },
    Remove { chall: Uuid },
}
