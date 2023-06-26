use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum Auth {
    #[serde(alias = "oauth")]
    OAuth {
        sub: String,
        provider: String,
        oauth_allow_token: String,
    },
    Pass {
        password: String,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "query_name", rename_all = "snake_case")]
pub enum UserQuery {
    #[serde(rename = "available")]
    CheckUsernameAvailability {
        name: String,
    },
    #[serde(rename = "create")]
    CreateNewUser {
        email: String,
        name: String,
        eligible: bool,
        admin: bool,
        auth: Auth,
    },
    #[serde(rename = "check_auth")]
    CheckUserAuth {
        id: Uuid,
        auth: Auth,
    },
    #[serde(rename = "update_auth")]
    UpdateUserAuth {
        id: Uuid,
        old_auth: Auth,
        new_auth: Auth,
    },
    #[serde(rename = "join")]
    JoinTeam {
        id: Uuid,
        auth: Auth,

        team_name: String,
        team_pass: String,
    },
    #[serde(rename = "get")]
    GetUser {
        id: Uuid,
    },
    #[serde(rename = "get_all")]
    GetAllUsers
}