use crate::payloads::*;

use incoming::sql::TeamQuery;
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::teams as queries;
use queries::{
    get_all_teams, get_team, get_team_by_name,
    create_team, update_team,
    check_team_auth,
};
use queries::{ TeamInput, NewTeamInput };

pub async fn handle(query: TeamQuery) -> Result<FromSql, FromSqlErr> {
    let success_res = match query {
        TeamQuery::GetAllTeams => FromSql::TeamArr(get_all_teams().await?),
        TeamQuery::GetTeam { id } => if let Some(team) = get_team(id).await? {
            FromSql::Team(team)
        } else {
            return Err(FromSqlErr::DoesNotExist(id))
        },
        TeamQuery::CheckTeamnameAvailability { name } => {
            let team = get_team_by_name(&name).await?;
            FromSql::Availability(team.is_none())
        },
        TeamQuery::CreateNewTeam { name, description, eligible, affiliation, password } => {
            use crate::passwords::*;

            let Ok(salt) = salt() else {
                return Err(FromSqlErr::OtherServerError("Failed to hash team password.".into()))
            };
            let Ok(hash) = argon2::hash_encoded(password.as_bytes(), &salt, &ARGON2_CONFIG) else {
                return Err(FromSqlErr::OtherServerError("Failed to hash team password.".into()))
            };

            FromSql::Team(
                create_team(NewTeamInput {
                    name,
                    description,
                    eligible,
                    affiliation,
                    hashed_password: hash
                }).await?
            )
        },
        TeamQuery::UpdateTeam { id, name, description, eligible, affiliation, password } => {
            if !check_team_auth(id, password).await? {
                return Err(FromSqlErr::DatabaseError)
            }

            FromSql::Team(
                update_team(TeamInput {
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