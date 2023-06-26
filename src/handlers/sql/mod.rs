mod prepared;

mod challs;
mod teams;
mod users;

use async_trait::async_trait;

use crate::payloads::incoming::ToSql;
use crate::payloads::outgoing::sql::{FromSql, FromSqlErr};
use crate::logging::*;

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToSql {
    type SuccessPayload = FromSql;
    type ErrorPayload = FromSqlErr;
    async fn handle(self) -> ResponseFrom<Self> {
        trace!("Handling SQL req");
        
        let return_payload = match self {
            ToSql::Chall(chall_query) => {
                debug!("SQL req classified as chall req");
                challs::handle(chall_query).await?
            },
            ToSql::Team(team_query) => {
                debug!("SQL req classified as team req");
                teams::handle(team_query).await?
            },
            ToSql::User(user_query) => {
                debug!("SQL req classified as user req");
                users::handle(user_query).await?
            },
        };
        Ok(return_payload)
    }
}

