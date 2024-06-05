use sqlx::query_as;
use uuid::Uuid;

use super::Ctx;
use crate::payloads::outgoing::sql::{ History, SimpleHistoryEntry };

// TODO --> Finish fixing this
pub async fn get_team_history(ctx: &mut Ctx, team_id: Uuid) -> Result<History, sqlx::Error> {
    struct HistoryEntry {
        time: chrono::NaiveDateTime,
        points_increase: i32,
    }
    

    let query = query_as!(
        HistoryEntry,
        r#"
            SELECT
                solved_at as time,
                COALESCE((SELECT points FROM challenges WHERE id = challenge_id), 0) as "points_increase!"
            FROM solve_successes
            WHERE team_id = $1
            ORDER BY solved_at ASC;
        "#,
        team_id,
    );

    let ret = query.fetch_all(ctx).await?;

    let mut history = History {
        team_id,
        entries: Vec::with_capacity(ret.len() + 1),
    };
    
    let mut points = 0;
    for entry in ret {
        points += entry.points_increase;
        history.entries.push(SimpleHistoryEntry {
            time: entry.time.and_utc().timestamp() as u64,
            points: points as u64,
        });

    }

    Ok(history)
}
