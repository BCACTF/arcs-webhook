use crate::handlers::sql::prepared::solves::first_blood_details;
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
use queries::SolveAttemptInput;

pub async fn handle(mut ctx: super::Ctx, query: SolveQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL solve req");

    let success_res = match query {
        SolveQuery::GetAllSolves => {
            debug!("SQL solve req classified as 'GetAllSolves' req");
            FromSql::SolveArr(get_all_solves(&mut ctx).await?)
        },
        SolveQuery::GetAllSolvesByChall { chall_id } => {
            debug!("SQL solve req classified as 'GetAllSolvesByChall<{chall_id}>' req");
            FromSql::SolveArr(get_solves_by_chall(&mut ctx, chall_id).await?)
        },
        SolveQuery::GetAllSolvesByTeam { team_id } => {
            debug!("SQL solve req classified as 'GetAllSolvesByTeam<{team_id}>' req");
            FromSql::SolveArr(get_solves_by_team(&mut ctx, team_id).await?)
        },
        SolveQuery::GetAllSolvesByUser { user_id } => {
            debug!("SQL solve req classified as 'GetAllSolvesByUser<{user_id}>' req");
            FromSql::SolveArr(get_solves_by_user(&mut ctx, user_id).await?)
        },
        SolveQuery::GetSolve { id } => {
            debug!("SQL solve req classified as 'GetSolve<{id}>' req");
            
            if let Some(solve) = get_solve(&mut ctx, id).await? {
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

            match user_is_on_team(&mut ctx, user_id, team_id).await? {
                DoesNotExist => return Err(FromSqlErr::DoesNotExist(user_id)),
                NotOnTeam => return Err(FromSqlErr::Auth),
                IsOnTeam => (),
            }

            if !check_user_auth(&mut ctx, user_id, user_auth).await? {
                return Err(FromSqlErr::Auth)
            }

            let solve = attempt_solve(
                &mut ctx,
                SolveAttemptInput { user_id, team_id, chall_id, flag_guess },
            ).await?;

            if solve.correct {
                if let Some(blood_details) = first_blood_details(&mut ctx, solve.id).await? {
                    use crate::payloads::incoming::discord::*;
                    use crate::handlers::Handle;

                    info!("First blood on {}! Sending discord message...", blood_details.chall.str());

                    let message = ToDiscord::Participant(ParticipantMessage::FirstBlood {
                        chall_name: blood_details.chall.string(),
                        team: blood_details.team.string(),
                        user: blood_details.user.string(),
                    });

                    if let Err(e) = message.handle().await {
                        error!("Sending the discord first blood message failed!");
                        debug!("Discord err: {e:?}");
                    }
                }
            }

            FromSql::Solve(solve)
        },

    };
    Ok(success_res)
}