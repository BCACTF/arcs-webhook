use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use uuid::Uuid;


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ChallIdentifier {
    CurrDeployedId(Uuid), // NOTE: This is at the top for higher deserialization priority. DON'T MOVE IT PLEASE.
    Folder(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToDeploy {
    Deploy {
        chall: ChallIdentifier,
        force_wipe: bool,
    },
    Poll { id: Uuid },
    Remove { chall: Uuid },
    ModifyMeta {
        id: Uuid,
        name: Option<String>,
        desc: Option<String>,
        points: Option<u64>,
        categories: Option<Vec<String>>,
        tags: Option<Option<Vec<String>>>,
        visible: Option<bool>,
    },
}
