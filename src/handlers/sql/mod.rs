mod prepared;

mod challs;
mod solves;
mod teams;
mod users;

use async_trait::async_trait;

use crate::payloads::incoming::ToSql;
use crate::payloads::outgoing::sql::{FromSql, FromSqlErr};
use crate::logging::*;

use super::{Handle, ResponseFrom};

use sqlx::{ pool::PoolConnection, Postgres };
type Ctx = PoolConnection<Postgres>;

pub use challs::{ get_chall_id_by_source_folder, get_chall_source_folder_by_id };

#[async_trait]
impl Handle for ToSql {
    type SuccessPayload = FromSql;
    type ErrorPayload = FromSqlErr;
    async fn handle(self) -> ResponseFrom<Self> {
        trace!("Handling SQL req");
        
        let sql_connection = crate::sql::connection().await?;
        debug!("Database connection acquired.");

        let return_payload = match self {
            ToSql::Chall(chall_query) => {
                debug!("SQL req classified as chall req");
                challs::handle(sql_connection, chall_query).await?
            },
            ToSql::Team(team_query) => {
                debug!("SQL req classified as team req");
                teams::handle(sql_connection, team_query).await?
            },
            ToSql::User(user_query) => {
                debug!("SQL req classified as user req");
                users::handle(sql_connection, user_query).await?
            },
            ToSql::Solve(solve_query) => {
                debug!("SQL req classified as solve req");
                solves::handle(sql_connection, solve_query).await?
            },
        };
        Ok(return_payload)
    }
}

