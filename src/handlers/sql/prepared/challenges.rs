use sqlx::{ query, query_as };
use uuid::Uuid;

use crate::sql;
use crate::payloads::{
    incoming::sql::Link,
    outgoing::sql::Chall,
};

pub async fn set_chall_updated(id: Uuid) -> Result<u64, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query!(
        r#"
            UPDATE challenges
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

pub async fn get_chall(id: Uuid) -> Result<Option<Chall>, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query_as!(
        Chall,
        r#"
            SELECT
                id,
                name as "name: _", description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder
            FROM challenges WHERE id = $1;
        "#,
        id,
    );
    query.fetch_optional(&mut sql_connection).await
}

pub async fn get_chall_by_source_folder(folder: &str) -> Result<Option<Chall>, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query_as!(
        Chall,
        r#"
            SELECT
                id,
                name as "name: _", description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder
            FROM challenges WHERE source_folder = $1;
        "#,
        folder,
    );
    query.fetch_optional(&mut sql_connection).await
}

pub async fn get_all_challs() -> Result<Vec<Chall>, sqlx::Error> {
    println!("tp1");
    let mut sql_connection = sql::connection().await?;
    println!("tp2");
    let query = query_as!(
        Chall,
        r#"
            SELECT
                id,
                name as "name: _", description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder
            FROM challenges;
        "#,
    );
    println!("tp3");
    query.fetch_all(&mut sql_connection).await
}


#[derive(Debug, Clone)]
pub struct ChallInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub points: Option<i32>,
    pub authors: Option<Vec<String>>,
    pub hints: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub links: Option<Vec<Link>>,
    pub visible: Option<bool>,
    pub source_folder: Option<String>,
}

pub async fn update_chall(id: Uuid, input: ChallInput) -> Result<Option<Chall>, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query!(
        r#"
            UPDATE challenges
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description),
                points = COALESCE($4, points),
                authors = COALESCE($5, authors),
                hints = COALESCE($6, hints),
                categories = COALESCE($7, categories),
                tags = COALESCE($8, tags),
                visible = COALESCE($9, visible),
                source_folder = COALESCE($10, source_folder)
            WHERE id = $1;
        "#,
        id,
        input.name: String,
        input.description,
        input.points,
        input.authors.as_deref(),
        input.hints.as_deref(),
        input.categories.as_deref(),
        input.tags.as_deref(),
        input.visible,
        input.source_folder,
    );
    let affected = query
        .execute(&mut sql_connection)
        .await?
        .rows_affected();

    if affected != 1 { return Ok(None) }

    
    set_chall_updated(id).await?;

    let Some(output) = get_chall(id).await? else {
        return Err(sqlx::Error::RowNotFound);
    };
    Ok(Some(output))
}


#[derive(Debug, Clone)]
pub struct NewChallInput {
    pub name: String,
    pub description: String,
    pub points: i32,
    pub authors: Vec<String>,
    pub hints: Vec<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub links: Vec<Link>,
    pub visible: bool,
    pub source_folder: String,
}

pub async fn create_chall(input: NewChallInput) -> Result<Chall, sqlx::Error> {
    let mut sql_connection = sql::connection().await?;
    let query = query!(
        r#"
            INSERT INTO challenges (
                name, description, points,
                authors, hints, categories, tags,
                visible, source_folder
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9);
        "#,
        input.name: String,
        input.description,
        input.points,
        &input.authors,
        &input.hints,
        &input.categories,
        &input.tags,
        input.visible,
        &input.source_folder
    );
    query.execute(&mut sql_connection).await?;

    let Some(output) = get_chall_by_source_folder(&input.source_folder).await? else {
        return Err(sqlx::Error::RowNotFound);
    };

    set_chall_updated(output.id).await?;
    Ok(output)
}
