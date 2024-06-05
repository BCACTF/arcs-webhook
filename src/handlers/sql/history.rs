use crate::handlers::sql::prepared::teams::get_top_teams;
use crate::logging::*;
use crate::payloads::*;

use incoming::sql::HistoryQuery;
use outgoing::sql::History;
use outgoing::sql::{FromSql, FromSqlErr};
use uuid::Uuid;

// use crate::handlers::sql::attempts as queries;

use crate::handlers::sql::prepared::history::get_team_history;


async fn get_team_histories(ctx: &mut super::Ctx, team_ids: Vec<Uuid>) -> Result<Vec<History>, FromSqlErr> {
    let mut histories = Vec::new();
    for team_id in team_ids {
        histories.push(get_team_history(ctx, team_id).await?);
    }
    Ok(histories)
}

pub async fn handle(mut ctx: super::Ctx, query: HistoryQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL attempt req");

    let success_res = match query {
        HistoryQuery::GetTeamHistories { team_ids } => {
            debug!("SQL attempt req classified as 'GetTeamHistories' req");
            FromSql::HistoryArr(get_team_histories(&mut ctx, team_ids).await?)
        },
        HistoryQuery::GetTopTeamHistories { limit } => {
            debug!("SQL attempt req classified as 'GetTopTeamHistories' req");
            let top_teams = get_top_teams(&mut ctx, limit).await?;
            FromSql::HistoryArr(get_team_histories(&mut ctx, top_teams).await?)
        }
    };
    Ok(success_res)
}
