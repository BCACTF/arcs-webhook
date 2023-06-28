use crate::logging::*;
use crate::payloads::*;

use incoming::sql::ChallQuery;
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::challenges as queries;
use queries::{
    get_all_challs, get_chall,
    create_chall, update_chall,
};
use queries::{ ChallInput, NewChallInput };

pub async fn handle(mut ctx: super::Ctx, query: ChallQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL chall req");

    let success_res = match query {
        ChallQuery::GetAllChallenges => {
            debug!("SQL chall req classified as 'GetAllChallenges' req");
            FromSql::ChallArr(get_all_challs(&mut ctx).await?)
        },
        ChallQuery::GetChallenge { id } => {
            debug!("SQL chall req classified as 'GetChallenge<{id}>' req");

            if let Some(chall) = get_chall(&mut ctx, id).await? {
                FromSql::Chall(chall)
            } else {
                return Err(FromSqlErr::DoesNotExist(id))
            }
        },
        ChallQuery::CreateChallenge {
            name, description, points,
            authors, hints, categories, tags, links,
            visible, source_folder,
        } => {
            debug!("SQL chall req classified as 'CreateChallenge<`{name}`>' req");
            FromSql::Chall(create_chall(&mut ctx, NewChallInput {
                name, description, points,
                authors, hints, categories, tags, links,
                visible, source_folder
            }).await?)
        },
        ChallQuery::UpdateChallenge {
            id,
            name, description, points,
            authors, hints, categories, tags, links,
            visible, source_folder
        } => {
            debug!("SQL chall req classified as 'UpdateChallenge<`{id}`>' req");

            let opt_chall = update_chall(&mut ctx, id, ChallInput {
                name, description, points,
                authors, hints, categories, tags, links,
                visible, source_folder,
            }).await?;

            if let Some(chall) = opt_chall {
                FromSql::Chall(chall)
            } else {
                return Err(FromSqlErr::DoesNotExist(id));
            }
        }
    };
    Ok(success_res)
}