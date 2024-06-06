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

        TeamQuery::CheckTeamnameAvailability { name } => {
            let display_name = shortened(&name, 13);
            debug!("SQL team req classified as 'CheckTeamnameAvailability<`{display_name}`>' req");

            let team = get_team_by_name(&mut ctx, &name).await?;
            FromSql::Availability(team.is_none())
        },
        TeamQuery::CreateNewTeam {
            name, description, eligible, affiliation,
            password,
            initial_user, user_auth,
        } => {
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

            let Some(user) = super::prepared::users::get_user(&mut ctx, initial_user).await? else {
                warn!("Initial user {initial_user} does not exist, tried to create a team");
                return Err(FromSqlErr::DoesNotExist(initial_user));
            };
            if user.team_id.is_some() {
                warn!("Initial user {initial_user} already on a team, tried to create a team");
                return Err(FromSqlErr::OtherServerError(format!("{} already on team", user.id).into()));
            }

            if !super::prepared::users::check_user_auth(&mut ctx, initial_user, user_auth).await? {
                warn!("Initial user {initial_user} failed to auth");
                return Err(FromSqlErr::Auth);
            }


            let team = create_team(&mut ctx, NewTeamInput {
                name,
                description,
                eligible,
                affiliation,
                hashed_password: hash
            }).await?;


            super::prepared::users::set_user_team(&mut ctx, initial_user, team.id, None).await?;


            let Some(team) = get_team(&mut ctx, team.id).await? else {
                error!("Couldn't find team {:?} ({}) which was just created", team.name, team.id);
                return Err(FromSqlErr::OtherServerError("Failed to create team".into()))
            };

            FromSql::Team(team)
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