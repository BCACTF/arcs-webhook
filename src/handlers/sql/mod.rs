mod prepared;

mod challs;
mod teams;
mod users;

use async_trait::async_trait;

use crate::payloads::incoming::ToSql;
use crate::payloads::outgoing::sql::{FromSql, FromSqlErr};

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToSql {
    type SuccessPayload = FromSql;
    type ErrorPayload = FromSqlErr;
    async fn handle(self) -> ResponseFrom<Self> {
        let return_payload = match self {
            ToSql::Chall(chall_query) => challs::handle(chall_query).await?,
            ToSql::Team(team_query) => teams::handle(team_query).await?,
            ToSql::User(user_query) => users::handle(user_query).await?,
        };
        Ok(return_payload)
    }
}

