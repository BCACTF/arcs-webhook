mod prepared;

mod challs;
mod solves;
mod teams;
mod users;
mod attempts;
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
                match challs::handle(sql_connection, chall_query).await {
                    Ok(return_payload) => return_payload,
                    Err(e) => {
                        debug!("Challs SQL error: {e:?}");
                        return Err(e);
                    }
                }
            },
            ToSql::Team(team_query) => {
                debug!("SQL req classified as team req");
                match teams::handle(sql_connection, team_query).await {
                    Ok(return_payload) => return_payload,
                    Err(e) => {
                        debug!("Teams SQL error: {e:?}");
                        return Err(e);
                    }
                }
            },
            ToSql::User(user_query) => {
                debug!("SQL req classified as user req");
                match users::handle(sql_connection, user_query).await {
                    Ok(return_payload) => return_payload,
                    Err(e) => {
                        debug!("Users SQL error: {e:?}");
                        return Err(e);
                    }
                }
            },
            ToSql::Solve(solve_query) => {
                debug!("SQL req classified as solve req");
                match solves::handle(sql_connection, solve_query).await {
                    Ok(return_payload) => return_payload,
                    Err(e) => {
                        debug!("Solve SQL error: {e:?}");
                        return Err(e);
                    }
                }
            },
            ToSql::Attempt(attempt_query ) => {
                debug!("SQL req classified as attempt req");
                match attempts::handle(sql_connection, attempt_query).await {
                    Ok(return_payload) => return_payload,
                    Err(e) => {
                        debug!("Attempts SQL error: {e:?}");
                        return Err(e);
                    }
                }
            }
        };
        Ok(return_payload)
    }
}

