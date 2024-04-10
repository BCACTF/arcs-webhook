use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__type", rename_all = "snake_case", content = "params")]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__query_name", rename_all = "snake_case", content = "params")]
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
    #[serde(rename = "promote")]
    Promote {
        admin_id: Uuid,
        admin_auth: Auth,
        user_to_promote: Uuid,
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