use crate::payloads::*;

use incoming::sql::ChallQuery;
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::challenges as queries;
use queries::{
    get_all_challs, get_chall,
    create_chall, update_chall,
};
use queries::{ ChallInput, NewChallInput };

pub async fn handle(query: ChallQuery) -> Result<FromSql, FromSqlErr> {
    let success_res = match query {
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
    };
    Ok(success_res)
}