use std::borrow::Cow;

use serde::Serialize;
use uuid::Uuid;

use crate::handlers::{OutgoingErr, sql::prepared::{challenges::Chall, teams::Team}};

// TODO: Fix this
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "__type", rename_all = "snake_case", content = "data")]
pub enum FromSql {
    Chall(Chall),
    ChallArr(Vec<Chall>),
    
    Team(Team),
    TeamArr(Vec<Team>),
    
    // User(User),
    // UserArr(),
    
    // Solves(),

    Availability(bool),
}

#[derive(Debug, Clone, Serialize)]
pub enum FromSqlErr {
    OtherServerError(Cow<'static, str>),
    DatabaseError,
    Auth,
    DoesNotExist(Uuid),
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
                "err": "Team id does not exist.",
                "id": id,
            })),
        }
    }
    fn status_code(&self) -> u16 {
        match self {
            Self::OtherServerError(_) => 500,
            Self::DatabaseError => 500,
            Self::Auth => 403,
            Self::DoesNotExist(_) => 404,
        }
    }
}
