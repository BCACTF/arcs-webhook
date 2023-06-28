use crate::handlers::sql::prepared::users::check_user_auth;
use crate::logging::*;
use crate::payloads::*;

use incoming::sql::SolveQuery;
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::solves as queries;
use queries::{
    get_solve,
    get_solves_by_user, get_solves_by_team, get_solves_by_chall,
    get_all_solves,
    attempt_solve,
};
use queries::{ SolveAttemptInput };

pub async fn handle(query: SolveQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL solve req");

    let mut sql_connection = crate::sql::connection().await?;

    let success_res = match query {
        SolveQuery::GetAllSolves => {
            debug!("SQL solve req classified as 'GetAllSolves' req");
            FromSql::SolveArr(get_all_solves(&mut sql_connection).await?)
        },
        SolveQuery::GetAllSolvesByChall { chall_id } => {
            debug!("SQL solve req classified as 'GetAllSolvesByChall<{chall_id}>' req");
            FromSql::SolveArr(get_solves_by_chall(&mut sql_connection, chall_id).await?)
        },
        SolveQuery::GetAllSolvesByTeam { team_id } => {
            debug!("SQL solve req classified as 'GetAllSolvesByTeam<{team_id}>' req");
            FromSql::SolveArr(get_solves_by_team(&mut sql_connection, team_id).await?)
        },
        SolveQuery::GetAllSolvesByUser { user_id } => {
            debug!("SQL solve req classified as 'GetAllSolvesByUser<{user_id}>' req");
            FromSql::SolveArr(get_solves_by_user(&mut sql_connection, user_id).await?)
        },
        SolveQuery::GetSolve { id } => {
            debug!("SQL solve req classified as 'GetSolve<{id}>' req");
            
            if let Some(solve) = get_solve(&mut sql_connection, id).await? {
                FromSql::Solve(solve)
            } else {
                return Err(FromSqlErr::DoesNotExist(id))
            }
        },
        SolveQuery::AttemptSolve {
            user_id, team_id, chall_id,
            user_auth,
            flag_guess
         } => {
            debug!("SQL solve req classified as 'AttemptSolve' req");

            use super::prepared::users::{ user_is_on_team, UserIsOnTeamOutcome::* };

            match user_is_on_team(user_id, team_id).await? {
                UserDoesNotExist => return Err(FromSqlErr::DoesNotExist(user_id)),
                UserNotOnTeam => return Err(FromSqlErr::Auth),
                UserIsOnTeam => (),
            }

            if !check_user_auth(user_id, user_auth).await? {
                return Err(FromSqlErr::Auth)
            }

            FromSql::Solve(
                attempt_solve(
                    &mut sql_connection,
                    SolveAttemptInput { user_id, team_id, chall_id, flag_guess },
                ).await?
            )
        },

    };
    Ok(success_res)
}