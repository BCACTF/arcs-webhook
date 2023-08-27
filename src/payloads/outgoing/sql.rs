mod types;

use std::borrow::Cow;

use serde::Serialize;
use uuid::Uuid;

use crate::handlers::OutgoingErr;

// TODO: Fix this
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "__type", rename_all = "snake_case", content = "data")]
pub enum FromSql {
    Chall(Chall),
    ChallArr(Vec<Chall>),
    
    Team(Team),
    TeamArr(Vec<Team>),
    
    User(User),
    UserArr(Vec<User>),
    
    Solve(Solve),
    SolveArr(Vec<Solve>),

    Availability(bool),
    AuthStatus(bool),
}

#[derive(Debug, Clone, Serialize)]
pub enum FromSqlErr {
    OtherServerError(Cow<'static, str>),
    DatabaseError,
    Auth,
    DoesNotExist(Uuid),
    NameDoesNotExist(String),
    NameIsTaken(String),
}

impl From<sqlx::Error> for FromSqlErr {
    fn from(_: sqlx::Error) -> Self {
        Self::DatabaseError
    }
}

impl OutgoingErr for FromSqlErr {
    fn body(self) -> Result<serde_json::Value, String> {
        match self {
            Self::OtherServerError(description) => Ok(serde_json::json!({
                "err": "An unknown server error occured.",
                "info": description,
            })),
            Self::DatabaseError => Ok(serde_json::json!({
                "err": "Unexpected database error encountered.",
            })),
            Self::Auth => Ok(serde_json::json!({
                "err": "Unauthorized access.",
            })),
            Self::DoesNotExist(id) => Ok(serde_json::json!({
                "err": "This id does not exist.",
                "id": id,
            })),
            Self::NameDoesNotExist(name) => Ok(serde_json::json!({
                "err": "This name does not exist.",
                "name": name,
            })),
            Self::NameIsTaken(name) => Ok(serde_json::json!({
                "err": "A team with this name already exists.",
                "name": name,
            })),
        }
    }
    fn status_code(&self) -> u16 {
        match self {
            Self::OtherServerError(_) | Self::DatabaseError => 500,
            Self::DoesNotExist(_) | Self::NameDoesNotExist(_) => 404,
            Self::NameIsTaken(_) => 400,
            Self::Auth => 403,
        }
    }
}

pub use types::{ Chall, Solve, Team, User };

