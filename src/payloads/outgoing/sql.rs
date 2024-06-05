mod types;

use std::borrow::Cow;

use serde::Serialize;

use uuid::Uuid;

use crate::handlers::OutgoingErr;

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
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

    Attempts(Attempts),
    AttemptsArr(Vec<Attempts>),

    History(History),
    HistoryArr(Vec<History>),
}

#[derive(Debug, Clone, Serialize, schemars::JsonSchema)]
pub enum FromSqlErr {
    OtherServerError(Cow<'static, str>),
    DatabaseError,
    Auth,
    DoesNotExist(Uuid),
    NameDoesNotExist(String),
    NameIsTaken(String),
    RequestTooBig(u64, u64),
}

impl From<sqlx::Error> for FromSqlErr {
    fn from(e: sqlx::Error) -> Self {
        crate::logging::trace!("SQLX error: {e:?}");
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
            Self::RequestTooBig(size, limit) => Ok(serde_json::json!({
                "err": "Request too big (Check that your request is in size limits).",
                "size": size,
                "limit": limit,
            })),
        }
    }
    fn status_code(&self) -> u16 {
        match self {
            Self::OtherServerError(_) | Self::DatabaseError => 500,
            Self::RequestTooBig(_, _) => 413,
            Self::DoesNotExist(_) | Self::NameDoesNotExist(_) => 404,
            Self::Auth => 403,
            Self::NameIsTaken(_) => 400,
        }
    }
}

pub use types::{ Chall, Solve, Team, History, SimpleHistoryEntry, User, Attempts };
