use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum UserQuery {
    
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query_name", rename_all = "snake_case")]
pub enum TeamQuery {
    #[serde(rename = "available")]
    CheckTeamnameAvailability {
        name: String,
    },
    #[serde(rename = "create")]
    CreateNewTeam {
        name: String,
        description: String,
        eligible: bool,
        affiliation: Option<String>,
        password: String,
    },
    #[serde(rename = "create")]
    UpdateTeam {
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        eligible: Option<bool>,
        affiliation: Option<Option<String>>,
        password: String,
    },
    #[serde(rename = "get")]
    GetTeam {
        id: Uuid,
    },
    #[serde(rename = "get_all")]
    GetAllTeams
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkType {
    Nc,
    Web,
    Admin,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    #[serde(rename = "type")]
    link_type: LinkType,
    location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query_name", rename_all = "snake_case")]
pub enum ChallQuery {
    #[serde(rename = "create")]
    CreateChallenge {
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


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToSql {
    // User(UserQuery),
    Team(TeamQuery),
    Chall(ChallQuery),
    // Solve(SolveQuery),
}
