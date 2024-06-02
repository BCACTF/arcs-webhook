use sqlx::{ query_as, query };
use uuid::Uuid;

use super::Ctx;
use crate::payloads::outgoing::sql::Attempts;

// TODO --> Finish fixing this
pub async fn get_all_attempts_by_chall(ctx: &mut Ctx, chall_id: Uuid) -> Result<Attempts, sqlx::Error> {
    struct ChallAttempts {
        chall_id: Uuid,
        correct: Option<i64>,
        incorrect: Option<i64>,
    }
    
    let query = query_as!(
        ChallAttempts,
        r#"
            SELECT
                challenge_id as chall_id,
                SUM(CASE WHEN correct = TRUE THEN 1 ELSE 0 END) AS correct,
                SUM(CASE WHEN correct = FALSE THEN 1 ELSE 0 END) AS incorrect
            FROM solve_attempts
            WHERE challenge_id = $1
            GROUP BY challenge_id;
        "#,
        chall_id,
    );

    let ret = query.fetch_optional(ctx).await;

    match ret {
        Ok(Some(attempts)) => {
            Ok(Attempts {
                team_id: None,
                chall_id: Some(attempts.chall_id),
                correct: attempts.correct.unwrap_or(0) as u64,
                incorrect: attempts.incorrect.unwrap_or(0) as u64,
            })
        },
        Ok(None) => {
            Err(sqlx::Error::RowNotFound)
        }
        Err(e) => {
            Err(e)
        }
    }
}

// TODO --> Finish fixing this
pub async fn get_all_attempts_by_team(ctx: &mut Ctx, team_id: Uuid) -> Result<Attempts, sqlx::Error> {
    struct TeamAttempts {
        team_id: Uuid,
        correct: Option<i64>,
        incorrect: Option<i64>,
    }

    let query = query_as!(
        TeamAttempts,
        r#"
            SELECT
                team_id,
                SUM(CASE WHEN correct = TRUE THEN 1 ELSE 0 END) AS correct,
                SUM(CASE WHEN correct = FALSE THEN 1 ELSE 0 END) AS incorrect
            FROM solve_attempts
            WHERE team_id = $1
            GROUP BY team_id;
        "#,
        team_id,
    );

    let ret = query.fetch_optional(ctx).await;

    match ret {
        Ok(Some(attempts)) => {
            Ok(Attempts {
                team_id: Some(attempts.team_id),
                chall_id: None,
                correct: attempts.correct.unwrap_or(0) as u64,
                incorrect: attempts.incorrect.unwrap_or(0) as u64,
            })
        },
        Ok(None) => {
            Err(sqlx::Error::RowNotFound)
        },
        Err(e) => {
            Err(e)
        }
    }
}