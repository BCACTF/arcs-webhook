pub mod prepared;

use async_trait::async_trait;

use crate::payloads::incoming::ToSql;
use crate::payloads::incoming::sql::{ChallQuery, TeamQuery};
use crate::payloads::outgoing::sql::{FromSql, FromSqlErr};

use prepared::challenges::{
    get_all_challs, get_chall,
    create_chall, update_chall
};
use prepared::challenges::{ ChallInput, NewChallInput };


use prepared::teams::{
    get_all_teams, get_team, get_team_by_name,
    create_team, update_team,
    check_team_auth,
};
use prepared::teams::{ TeamInput, NewTeamInput };

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToSql {
    type SuccessPayload = FromSql;
    type ErrorPayload = FromSqlErr;
    async fn handle(self) -> ResponseFrom<Self> {
        let return_payload = match self {
            ToSql::Chall(chall_query) => match chall_query {
                ChallQuery::GetAllChallenges => FromSql::ChallArr(get_all_challs().await?),
                ChallQuery::GetChallenge { id } => FromSql::Chall(get_chall(id).await?),
                ChallQuery::CreateChallenge {
                    name, description, points,
                    authors, hints, categories, tags, links,
                    visible, source_folder,
                } => FromSql::Chall(create_chall(NewChallInput {
                    name, description, points,
                    authors, hints, categories, tags, links,
                    visible, source_folder
                }).await?),
                ChallQuery::UpdateChallenge {
                    id,
                    name, description, points,
                    authors, hints, categories, tags, links,
                    visible, source_folder
                } => FromSql::Chall(update_chall(id, ChallInput {
                    name, description, points,
                    authors, hints, categories, tags, links,
                    visible, source_folder,
                }).await?)
            },
            ToSql::Team(team_query) => match team_query {
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
                    use crate::passwords::*;

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
                _ => todo!(),
            },
        };
        Ok(return_payload)
    }
}

