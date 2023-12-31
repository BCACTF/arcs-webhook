use chrono::NaiveDateTime;
use sqlx::{ query, query_as };
use uuid::Uuid;

use super::Ctx;
use crate::payloads::outgoing::sql::{FromSqlErr, Team, ScoreEntry};


pub async fn set_team_updated(ctx: &mut Ctx, id: Uuid) -> Result<u64, sqlx::Error> {
    let query = query!(
        r#"
            UPDATE teams
            SET updated_at = DEFAULT
            WHERE id = $1;
        "#,
        id,
    );
    query
        .execute(ctx)
        .await
        .map(|res| res.rows_affected())
}



pub async fn get_team(ctx: &mut Ctx, id: Uuid) -> Result<Option<Team>, sqlx::Error> {

    let query = query_as!(
        Team,
        r#"
            SELECT
                id, name as "name: _", score,
                last_solve, eligible, affiliation
            FROM teams WHERE id = $1;
        "#,
        id,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_team_by_name(ctx: &mut Ctx, name: &str) -> Result<Option<Team>, sqlx::Error> {

    let query = query_as!(
        Team,
        r#"
            SELECT
                id, name as "name: _", score,
                last_solve, eligible, affiliation
            FROM teams WHERE name = $1;
        "#,
        name: String,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_all_teams(ctx: &mut Ctx) -> Result<Vec<Team>, sqlx::Error> {

    let query = query_as!(
        Team,
        r#"
            SELECT
                id, name as "name: _", score,
                last_solve, eligible, affiliation
            FROM teams;
        "#,
    );
    query.fetch_all(ctx).await
}

pub async fn get_team_batch(ctx: &mut Ctx, ids: &[Uuid]) -> Result<Vec<Team>, sqlx::Error> {
        let query = query_as!(
            Team,
            r#"
                SELECT
                    id, name as "name: _", score,
                    last_solve, eligible, affiliation
                FROM teams
                WHERE id IN (SELECT * FROM unnest($1::uuid[]));
            "#,
            ids
        );
        query.fetch_all(ctx).await
}

pub async fn get_top_teams(ctx: &mut Ctx, count: u32) -> Result<Vec<Uuid>, sqlx::Error> {
    let query = query!(
        r#"
            SELECT get_top_n_teams($1) as "id!";
        "#,
        count as i64,
    );
    let list = query
        .fetch_all(ctx).await?
        .into_iter().map(|row| row.id)
        .collect();
    Ok(list)
}


pub async fn get_team_score_history_batch(ctx: &mut Ctx, team_ids: &[Uuid], start_time: NaiveDateTime) -> Result<Vec<ScoreEntry>, sqlx::Error> {
    let get_initial_scores_query = query_as!(
        ScoreEntry,
        r#"
            SELECT
                team.id AS team_id,
                get_team_score_at(team.id, $2) AS "score!",
                $2 AS "time!"
            FROM teams as team
            WHERE team.id IN (SELECT * FROM unnest($1::uuid[]));
        "#,
        team_ids,
        start_time
    );

    let get_score_increments = query_as!(
        ScoreEntry,
        r#"
            SELECT
                solve.team_id AS team_id,
                (get_team_score_at(solve.team_id, solve.solved_at) + chall.points) AS "score!",
                solve.solved_at AS "time!"
            FROM solve_successes AS solve
            JOIN challenges AS chall ON solve.challenge_id = chall.id
            WHERE
                solve.team_id IN (SELECT * FROM unnest($1::uuid[])) AND
                solve.solved_at >= $2;
        "#,
        team_ids,
        start_time
    );

    let initial_scores = get_initial_scores_query.fetch_all(&mut *ctx).await?;
    let score_increments = get_score_increments.fetch_all(&mut *ctx).await?;

    let scores = initial_scores.into_iter().chain(score_increments.into_iter()).collect::<Vec<_>>();
    
    Ok(scores)
}


#[derive(Debug, Clone)]
pub struct NewTeamInput {
    pub name: String,
    pub description: String,
    pub eligible: bool,
    pub affiliation: Option<String>,
    pub hashed_password: String,
}


pub async fn create_team(ctx: &mut Ctx, input: NewTeamInput) -> Result<Team, sqlx::Error> {
    let query = query!(
        r#"
            INSERT INTO teams (name, description, eligible, affiliation, hashed_password)
            VALUES ($1, $2, $3, $4, $5);
        "#,
        input.name: String,
        input.description,
        input.eligible,
        input.affiliation,
        input.hashed_password,
    );
    query
        .execute(&mut *ctx)
        .await?;
    
    let team = get_team_by_name(&mut *ctx, &input.name).await?;
    let team = team.ok_or_else(|| sqlx::Error::RowNotFound)?;
    
    set_team_updated(ctx, team.id).await?;

    Ok(team)
}

#[derive(Debug, Clone)]
pub struct TeamInput {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub eligible: Option<bool>,
    pub affiliation: Option<Option<String>>,
}


pub async fn update_team(ctx: &mut Ctx, input: TeamInput) -> Result<Team, sqlx::Error> {


    let query = query!(
        r#"
            UPDATE teams
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                eligible = COALESCE($4, eligible)
            WHERE id = $1;
        "#,
        input.id,
        input.name: Option<String>,
        input.description,
        input.eligible,
    );

    let affected = if let Some(affiliation) = input.affiliation {

        let affiliation_query = query!(
            r#"
                UPDATE teams
                SET affiliation = $2
                WHERE id = $1;
            "#,
            input.id,
            affiliation
        );

        let affected = query
            .execute(&mut *ctx)
            .await?
            .rows_affected();

        if affected > 0 {
            affiliation_query
                .execute(&mut *ctx)
                .await?
                .rows_affected()
        } else {
            0
        }
    } else {
        query
            .execute(&mut *ctx)
            .await?
            .rows_affected()
    };
    if affected != 1 {
        return Err(sqlx::Error::RowNotFound)
    }


    if let Some(updated_team) = get_team(ctx, input.id).await? {
        Ok(updated_team)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}


pub enum CheckTeamAuthError {
    Sql(sqlx::Error),
    Hashing,
    NotFound(Uuid),
}

struct PasswordRow { hash: String }
impl From<sqlx::Error> for CheckTeamAuthError {
    fn from(value: sqlx::Error) -> Self {
        CheckTeamAuthError::Sql(value)
    }
}
impl From<CheckTeamAuthError> for FromSqlErr {
    fn from(value: CheckTeamAuthError) -> Self {
        match value {
            CheckTeamAuthError::Hashing => Self::OtherServerError("Failed to verify team password.".into()),
            CheckTeamAuthError::Sql(_) => Self::DatabaseError,
            CheckTeamAuthError::NotFound(id) => Self::DoesNotExist(id),
        }
    }
}

pub async fn check_team_auth(ctx: &mut Ctx, id: Uuid, password: String) -> Result<bool, CheckTeamAuthError> {

    let query = query_as!(
        PasswordRow,
        r#"
            SELECT hashed_password as hash FROM teams WHERE id = $1;
        "#,
        id,
    );
    let Some(row) = query.fetch_optional(ctx).await? else {
        return Err(CheckTeamAuthError::NotFound(id));
    };    
    argon2::verify_encoded(
        &row.hash,
        password.as_bytes(),
    ).map_err(|_| CheckTeamAuthError::Hashing)
}
