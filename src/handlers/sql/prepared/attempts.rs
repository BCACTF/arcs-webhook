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

pub async fn get_all_attempts(ctx: &mut Ctx) -> Result<Vec<Attempts>, sqlx::Error> {
    struct TeamAttempts {
        team_id: Uuid,
        correct: Option<i64>,
        incorrect: Option<i64>,
    }

    struct ChallAttempts {
        chall_id: Uuid,
        correct: Option<i64>,
        incorrect: Option<i64>,
    }

    let team_query = query_as!(
        TeamAttempts,
        r#"
            SELECT
                t.id as team_id,
                (SELECT COUNT(*) FROM solve_attempts AS att WHERE att.team_id = t.id AND att.correct) AS correct,
                (SELECT COUNT(*) FROM solve_attempts AS att WHERE att.team_id = t.id AND NOT att.correct) AS incorrect
            FROM teams as t;
        "#,
    );

    let chall_query = query_as!(
        ChallAttempts,
        r#"
            SELECT
                c.id as chall_id,
                (SELECT COUNT(*) FROM solve_attempts AS att WHERE att.challenge_id = c.id AND att.correct) AS correct,
                (SELECT COUNT(*) FROM solve_attempts AS att WHERE att.challenge_id = c.id AND NOT att.correct) AS incorrect
            FROM challenges as c;
        "#,
    );

    let team_attempts = team_query.fetch_all(&mut *ctx).await?;
    let chall_attempts = chall_query.fetch_all(&mut *ctx).await?;

    // Optimization!
    let expected_len = team_attempts.len() + chall_attempts.len();

    let team_attempts = team_attempts.into_iter().map(|team| {
        match (team.correct, team.incorrect) {
            (Some(correct), Some(incorrect)) => {
                Ok(Attempts {
                    team_id: Some(team.team_id),
                    chall_id: None,
                    correct: correct as u64,
                    incorrect: incorrect as u64,
                })
            },
            _ => {
                Err(sqlx::Error::RowNotFound)
            }
        }
    });

    let chall_attempts = chall_attempts.into_iter().map(|chall| {
        match (chall.correct, chall.incorrect) {
            (Some(correct), Some(incorrect)) => {
                Ok(Attempts {
                    team_id: None,
                    chall_id: Some(chall.chall_id),
                    correct: correct as u64,
                    incorrect: incorrect as u64,
                })
            },
            _ => {
                Err(sqlx::Error::RowNotFound)
            }
        }
    });

    team_attempts
        .chain(chall_attempts)
        .try_fold(
            Vec::with_capacity(expected_len),
            |mut output, attempts_obj| {
                match attempts_obj {
                    Ok(attempts) => {
                        output.push(attempts);
                        Ok(output)
                    },
                    Err(e) => {
                        Err(e)
                    }
                }
            },
        )
}
