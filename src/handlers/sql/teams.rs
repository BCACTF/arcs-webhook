use crate::payloads::*;
use crate::logging::*;

use incoming::sql::TeamQuery;
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::teams as queries;
use queries::{
    get_all_teams, get_team, get_team_by_name,
    create_team, update_team,
    check_team_auth,
};
use queries::{ TeamInput, NewTeamInput };

pub async fn handle(mut ctx: super::Ctx, query: TeamQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL team req");
        
    
    let success_res = match query {
        TeamQuery::GetAllTeams => {
            debug!("SQL team req classified as 'GetAllTeams' req");
            FromSql::TeamArr(get_all_teams(&mut ctx).await?)
        },
        TeamQuery::GetTeam { id } => {
            debug!("SQL team req classified as 'GetTeam<{id}>' req");
            if let Some(team) = get_team(&mut ctx, id).await? {
                FromSql::Team(team)
            } else {
                return Err(FromSqlErr::DoesNotExist(id))
            }
        },
        TeamQuery::GetTopTeams { limit } => {
            debug!("SQL team req classified as 'GetTopTeams<{limit}>' req");

            // Cap here to prevent server from being overloaded by a
            // badly-written client
            if limit > 100 {
                return Err(FromSqlErr::RequestTooBig(limit as u64, 100))
            }

            let top_team_ids = queries::get_top_teams(&mut ctx, limit).await?;

            FromSql::TeamArr(queries::get_team_batch(&mut ctx, &top_team_ids).await?)
        },
        TeamQuery::GetTopTeamsScoreHistory { limit, start_time } => {
            debug!("SQL team req classified as 'GetTopTeamsScoreHistory<{limit}>' req");

            // Cap here to prevent server from being overloaded by a
            // badly-written client
            if limit > 100 {
                return Err(FromSqlErr::RequestTooBig(limit as u64, 100))
            }
            
            let top_team_ids = queries::get_top_teams(&mut ctx, limit).await?;

            let team_score_history = queries::get_team_score_history_batch(&mut ctx, &top_team_ids, start_time).await?;
            FromSql::TeamScoreHistoryArray(team_score_history)
        },


        TeamQuery::CheckTeamnameAvailability { name } => {
            let display_name = shortened(&name, 13);
            debug!("SQL team req classified as 'CheckTeamnameAvailability<`{display_name}`>' req");

            let team = get_team_by_name(&mut ctx, &name).await?;
            FromSql::Availability(team.is_none())
        },
        TeamQuery::CreateNewTeam { name, description, eligible, affiliation, password } => {
            let display_name = shortened(&name, 13);
            let display_affil = affiliation.as_ref().map(|affil| shortened(affil, 13));
            debug!("SQL team req classified as 'CreateNewTeam<`{display_name}` of {display_affil:?}>' req");

            use crate::passwords::*;

            let Ok(salt) = salt() else {
                return Err(FromSqlErr::OtherServerError("Failed to hash team password.".into()))
            };
            let Ok(hash) = argon2::hash_encoded(password.as_bytes(), &salt, &ARGON2_CONFIG) else {
                return Err(FromSqlErr::OtherServerError("Failed to hash team password.".into()))
            };

            let team_already_exists = get_team_by_name(&mut ctx, &name).await?.is_some();
            if team_already_exists { return Err(FromSqlErr::NameIsTaken(name)); }

            FromSql::Team(
                create_team(&mut ctx, NewTeamInput {
                    name,
                    description,
                    eligible,
                    affiliation,
                    hashed_password: hash
                }).await
                .map_err(|err| { warn!("{err:?}"); err })?
            )
        },
        TeamQuery::UpdateTeam { id, name, description, eligible, affiliation, password } => {
            debug!("SQL team req classified as 'UpdateTeam<{id}>' req");

            if !check_team_auth(&mut ctx, id, password).await? {
                return Err(FromSqlErr::DatabaseError)
            }

            FromSql::Team(
                update_team(&mut ctx, TeamInput {
                    id,
                    name,
                    description,
                    eligible,
                    affiliation,
                }).await?
            )
        },
    };
    Ok(success_res)
}