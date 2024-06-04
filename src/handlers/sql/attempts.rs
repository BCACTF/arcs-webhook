use crate::logging::*;
use crate::payloads::*;

use incoming::sql::AttemptQuery;
use outgoing::sql::{FromSql, FromSqlErr};

// use crate::handlers::sql::attempts as queries;

use crate::handlers::sql::prepared::attempts::{
    get_all_attempts_by_chall,
    get_all_attempts_by_team,
    get_all_attempts,
};

pub async fn handle(mut ctx: super::Ctx, query: AttemptQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL attempt req");

    let success_res = match query {
        AttemptQuery::GetAllAttemptsByChall { chall_id } => {
            debug!("SQL attempt req classified as 'GetAllAttemptsByChall' req");
            FromSql::Attempts(get_all_attempts_by_chall(&mut ctx, chall_id).await?)
        },
        AttemptQuery::GetAllAttemptsByTeam { team_id } => {
            debug!("SQL attempt req classified as 'GetAllAttemptsByTeam<{team_id}>' req");
            FromSql::Attempts(get_all_attempts_by_team(&mut ctx, team_id).await?)
        },
        AttemptQuery::GetAllAttempts => {
            debug!("SQL attempt req classified as 'GetAllAttempts' req");
            FromSql::AttemptsArr(get_all_attempts(&mut ctx).await?)
        },
    };
    Ok(success_res)
}