use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Nc,
    Web,
    Admin,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Link {
    #[serde(rename = "type")]
    pub link_type: LinkType,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "query_name", rename_all = "snake_case")]
pub enum ChallQuery {
    #[serde(rename = "create")]
    CreateChallenge {
        id: Option<Uuid>,
        
        name: String,
        description: String,
        points: i32,
        authors: Vec<String>,
        hints: Vec<String>,
        categories: Vec<String>,
        tags: Vec<String>,
        links: Vec<Link>,

        visible: bool,
        source_folder: String,

        flag: String,
    },
    #[serde(rename = "update")]
    UpdateChallenge {
        id: Uuid,

        name: Option<String>,
        description: Option<String>,
        points: Option<i32>,
        authors: Option<Vec<String>>,
        hints: Option<Vec<String>>,
        categories: Option<Vec<String>>,
        tags: Option<Vec<String>>,
        links: Option<Vec<Link>>,

        visible: Option<bool>,
        source_folder: Option<String>,
    },
    #[serde(rename = "get")]
    GetChallenge {
        id: Uuid,
    },
    #[serde(rename = "get_all")]
    GetAllChallenges,
}
