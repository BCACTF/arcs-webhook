use sqlx::{ query, query_as };
use uuid::Uuid;

use crate::{sql, payloads::outgoing::sql::{FromSqlErr, Team}};

pub async fn set_team_updated(id: Uuid) -> Result<u64, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query!(
        r#"
            UPDATE teams
            SET updated_at = DEFAULT
            WHERE id = $1;
        "#,
        id,
    );
    query
        .execute(&mut sql_connection)
        .await
        .map(|res| res.rows_affected())
}



pub async fn get_team(id: Uuid) -> Result<Option<Team>, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query_as!(
        Team,
        r#"
            SELECT
                id, name as "name: _",
                last_solve, eligible, affiliation
            FROM teams WHERE id = $1;
        "#,
        id,
    );
    query.fetch_optional(&mut sql_connection).await
}

pub async fn get_team_by_name(name: &str) -> Result<Option<Team>, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query_as!(
        Team,
        r#"
            SELECT
                id, name as "name: _",
                last_solve, eligible, affiliation
            FROM teams WHERE name = $1;
        "#,
        name: String,
    );
    query.fetch_optional(&mut sql_connection).await
}

pub async fn get_all_teams() -> Result<Vec<Team>, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query_as!(
        Team,
        r#"
            SELECT
                id, name as "name: _",
                last_solve, eligible, affiliation
            FROM teams;
        "#,
    );
    query.fetch_all(&mut sql_connection).await
}


#[derive(Debug, Clone)]
pub struct NewTeamInput {
    pub name: String,
    pub description: String,
    pub eligible: bool,
    pub affiliation: Option<String>,
    pub hashed_password: String,
}


pub async fn create_team(input: NewTeamInput) -> Result<Team, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;


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
        .execute(&mut sql_connection)
        .await?;
    
    let team = get_team_by_name(&input.name).await?;
    let team = team.ok_or_else(|| sqlx::Error::RowNotFound)?;
    
    set_team_updated(team.id).await?;

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


pub async fn update_team(input: TeamInput) -> Result<Team, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;

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
            .execute(&mut sql_connection)
            .await?
            .rows_affected();

        if affected > 0 {
            affiliation_query
                .execute(&mut sql_connection)
                .await?
                .rows_affected()
        } else {
            0
        }
    } else {
        query
            .execute(&mut sql_connection)
            .await?
            .rows_affected()
    };
    if affected != 1 {
        return Err(sqlx::Error::RowNotFound)
    }


    if let Some(updated_team) = get_team(input.id).await? {
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

pub async fn check_team_auth(id: Uuid, password: String) -> Result<bool, CheckTeamAuthError> {
    let mut sql_connection = sql::connection().await?;
    let query = query_as!(
        PasswordRow,
        r#"
            SELECT hashed_password as hash FROM teams WHERE id = $1;
        "#,
        id,
    );
    let Some(row) = query.fetch_optional(&mut sql_connection).await? else {
        return Err(CheckTeamAuthError::NotFound(id));
    };    
    argon2::verify_encoded(
        &row.hash,
        password.as_bytes(),
    ).map_err(|_| CheckTeamAuthError::Hashing)
}
