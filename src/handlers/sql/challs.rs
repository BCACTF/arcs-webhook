use crate::logging::*;
use crate::payloads::*;

use incoming::sql::ChallQuery;
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::challenges as queries;
use super::prepared::challenges::get_chall_by_source_folder;
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
            id,
            name, description, points,
            authors, hints, categories, tags, links,
            visible, source_folder, flag
        } => {
            debug!("SQL chall req classified as 'CreateChallenge<`{name}`>' req");
            FromSql::Chall(create_chall(&mut ctx, NewChallInput {
                id,
                name, description, points,
                authors, hints, categories, tags, links,
                visible, source_folder, flag,
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

pub async fn get_chall_id_by_source_folder(source_folder: &str) -> Result<Option<uuid::Uuid>, std::borrow::Cow<'static, str>> {
    let Ok(mut sql_connection) = crate::sql::connection().await else {
        return Err("Failed to get db connection".into())
    };

    let Ok(id) = get_chall_by_source_folder(&mut sql_connection, source_folder).await else {
        return Err("Failed to check for challenge".into())
    };

    Ok(id.map(|c| c.id))
}
