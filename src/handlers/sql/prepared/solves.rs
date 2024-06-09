use sqlx::{ query_as, query };
use uuid::Uuid;

use super::Ctx;
use crate::payloads::outgoing::sql::Solve;


pub async fn get_solve(ctx: &mut Ctx, id: Uuid) -> Result<Option<Solve>, sqlx::Error> {
    let query = query_as!(
        Solve,
        r#"
            SELECT
                attempt.id, attempt.user_id, attempt.team_id, attempt.challenge_id AS chall_id,
                attempt.correct, attempt.inserted_at AS time,
                (success.id IS NOT NULL) AS "counted!"
            FROM solve_attempts AS attempt
                LEFT JOIN solve_successes AS success ON success.attempt_id = attempt.id
            WHERE attempt.id = $1;
        "#,
        id,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_solves_by_team(ctx: &mut Ctx, team_id: Uuid) -> Result<Vec<Solve>, sqlx::Error> {
    let query = query_as!(
        Solve,
        r#"
            SELECT
                attempt.id, attempt.user_id, attempt.team_id, attempt.challenge_id AS chall_id,
                attempt.correct, attempt.inserted_at AS time,
                (success.id IS NOT NULL) AS "counted!"
            FROM solve_attempts AS attempt
                LEFT JOIN solve_successes AS success ON success.attempt_id = attempt.id
            WHERE attempt.team_id = $1;
        "#,
        team_id,
    );
    query.fetch_all(ctx).await
}

pub async fn get_solves_by_chall(ctx: &mut Ctx, chall_id: Uuid) -> Result<Vec<Solve>, sqlx::Error> {
    let query = query_as!(
        Solve,
        r#"
            SELECT
                attempt.id, attempt.user_id, attempt.team_id, attempt.challenge_id AS chall_id,
                attempt.correct, attempt.inserted_at AS time,
                (success.id IS NOT NULL) AS "counted!"
            FROM solve_attempts AS attempt
                LEFT JOIN solve_successes AS success ON success.attempt_id = attempt.id
            WHERE attempt.challenge_id = $1;
        "#,
        chall_id,
    );
    query.fetch_all(ctx).await
}

pub async fn get_solves_by_user(ctx: &mut Ctx, user_id: Uuid) -> Result<Vec<Solve>, sqlx::Error> {
    let query = query_as!(
        Solve,
        r#"
            SELECT
                attempt.id, attempt.user_id, attempt.team_id, attempt.challenge_id AS chall_id,
                attempt.correct, attempt.inserted_at AS time,
                (success.id IS NOT NULL) AS "counted!"
            FROM solve_attempts AS attempt
                LEFT JOIN solve_successes AS success ON success.attempt_id = attempt.id
            WHERE attempt.user_id = $1;
        "#,
        user_id,
    );
    query.fetch_all(ctx).await
}

pub async fn get_all_solves(ctx: &mut Ctx) -> Result<Vec<Solve>, sqlx::Error> {
    let query = query_as!(
        Solve,
        r#"
            SELECT
                attempt.id AS "id!",
                attempt.user_id AS "user_id!", attempt.team_id AS "team_id!", attempt.challenge_id AS "chall_id!",
                attempt.correct AS "correct!", attempt.inserted_at AS "time!",
                (success.id IS NOT NULL) AS "counted!"
            FROM solve_attempts AS attempt
                LEFT JOIN solve_successes AS success ON success.attempt_id = attempt.id
            WHERE attempt.id IS NOT NULL;
        "#,
    );
    query.fetch_all(ctx).await
}


#[derive(Debug, Clone)]
pub struct SolveAttemptInput {
    pub user_id: Uuid,
    pub team_id: Uuid,
    pub chall_id: Uuid,
    pub flag_guess: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdRow { id: Option<Uuid> }
pub async fn attempt_solve(ctx: &mut Ctx, input: SolveAttemptInput) -> Result<Solve, sqlx::Error> {
    let query = query_as!(
        IdRow,
        r#"
            SELECT id FROM do_solve_attempt($1, $2, $3, $4) as (id uuid, guess_correct bool, already_solved bool);
        "#,
        input.user_id,
        input.team_id,
        input.chall_id,
        input.flag_guess,
    );

    let attempt_id = query
        .fetch_optional(&mut *ctx)
        .await?
        .and_then(|id_row| id_row.id)
        .ok_or(sqlx::Error::RowNotFound)?;
    
    get_solve(ctx, attempt_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

use crate::sql::CiText;

#[derive(Debug, Clone)]
pub struct FirstBloodInfo { pub chall: CiText, pub user: CiText, pub team: CiText }
pub async fn first_blood_details(ctx: &mut Ctx, solve_id: Uuid) -> Result<Option<FirstBloodInfo>, sqlx::Error> {
    let query = query_as!(
        FirstBloodInfo,
        r#"
            SELECT
                chall.name AS "chall: _",
                users.name AS "user: _",
                team.name AS "team: _"
            FROM solve_attempts AS attempt
                LEFT JOIN challenges AS chall ON chall.id = attempt.challenge_id
                LEFT JOIN teams AS team ON team.id = attempt.team_id
                LEFT JOIN users AS users ON users.id = attempt.user_id
            WHERE
                attempt.id = $1 AND
                attempt.correct AND
                chall.visible AND
                (
                    SELECT
                        att.id AS att_id
                    FROM solve_attempts AS att
                        WHERE att.challenge_id = chall.id AND att.correct
                    ORDER BY att.inserted_at LIMIT 1
                ) = $1;
        "#,
        solve_id,
    );
    query.fetch_optional(ctx).await
}

pub async fn clear_all_solves_for_challenge(ctx: &mut Ctx, chall_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
    let query = query!(
        r#"
            SELECT delete_solves_for_challenge($1) as "id!";
        "#,
        chall_id,
    );

    let deleted = query
        .fetch_all(ctx)
        .await?
        .into_iter()
        .map(|record| record.id)
        .collect();

    Ok(deleted)
}
