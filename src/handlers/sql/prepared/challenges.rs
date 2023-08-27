use sqlx::{ query, query_as };
use uuid::Uuid;

use super::Ctx;
use crate::payloads::{
    incoming::sql::Link,
    outgoing::sql::Chall,
};

pub async fn set_chall_updated(ctx: &mut Ctx, id: Uuid) -> Result<u64, sqlx::Error> {
    let query = query!(
        r#"
            UPDATE challenges
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

pub async fn get_chall(ctx: &mut Ctx, id: Uuid) -> Result<Option<Chall>, sqlx::Error> {
    let query = query_as!(
        Chall,
        r#"
            SELECT
                challenges.id,
                name as "name: _", description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder,
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'nc'    ), ARRAY[]::text[]) as "links_nc!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'web'   ), ARRAY[]::text[]) as "links_web!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'admin' ), ARRAY[]::text[]) as "links_admin!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'static'), ARRAY[]::text[]) as "links_static!"
            FROM challenges
                LEFT JOIN challenge_links as links ON links.challenge_id = challenges.id
            WHERE challenges.id = $1
            GROUP BY challenges.id;
        "#,
        id,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_chall_by_source_folder(ctx: &mut Ctx, folder: &str) -> Result<Option<Chall>, sqlx::Error> {
    let query = query_as!(
        Chall,
        r#"
            SELECT
                challenges.id,
                name as "name: _", description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder,
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'nc'    ), ARRAY[]::text[]) as "links_nc!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'web'   ), ARRAY[]::text[]) as "links_web!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'admin' ), ARRAY[]::text[]) as "links_admin!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'static'), ARRAY[]::text[]) as "links_static!"
            FROM challenges
                LEFT JOIN challenge_links as links ON links.challenge_id = challenges.id
            WHERE source_folder = $1
            GROUP BY challenges.id;
        "#,
        folder,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_all_challs(ctx: &mut Ctx) -> Result<Vec<Chall>, sqlx::Error> {
    let query = query_as!(
        Chall,
        r#"
            SELECT
                challenges.id,
                name as "name: _", description, points,
                authors, hints, categories, tags,
                solve_count, visible, source_folder,
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'nc'    ), ARRAY[]::text[]) as "links_nc!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'web'   ), ARRAY[]::text[]) as "links_web!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'admin' ), ARRAY[]::text[]) as "links_admin!",
                COALESCE(array_agg(links.url) FILTER (WHERE links.type = 'static'), ARRAY[]::text[]) as "links_static!"
            FROM challenges
                LEFT JOIN challenge_links as links ON links.challenge_id = challenges.id
            GROUP BY challenges.id;
        "#,
    );
    query.fetch_all(ctx).await
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

pub async fn update_chall(ctx: &mut Ctx, id: Uuid, input: ChallInput) -> Result<Option<Chall>, sqlx::Error> {
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
        .execute(&mut *ctx)
        .await?
        .rows_affected();

    if affected != 1 { return Ok(None) }

    if let Some(links) = input.links {
        set_chall_links(&mut *ctx, id, links).await?;
    }
    set_chall_updated(&mut *ctx, id).await?;

    let Some(output) = get_chall(ctx, id).await? else {
        return Err(sqlx::Error::RowNotFound);
    };

    Ok(Some(output))
}


pub async fn set_chall_links(ctx: &mut Ctx, id: Uuid, links: Vec<Link>) -> Result<(), sqlx::Error> {
    let (web_links, nc_links, admin_links, static_links) = {
        let mut web_links = vec![];
        let mut nc_links = vec![];
        let mut admin_links = vec![];
        let mut static_links = vec![];

        for link in links {
            use crate::payloads::incoming::sql::LinkType::*;
            match link.link_type {
                Web => web_links.push(link.location),
                Nc => nc_links.push(link.location),
                Admin => admin_links.push(link.location),
                Static => static_links.push(link.location),
            }
        }
        (web_links, nc_links, admin_links, static_links)
    };

    let query = query!(
        r#"
        SELECT replace_challenge_links($1, $2, $3, $4, $5);
        "#,
        id,
        &web_links,
        &nc_links,
        &admin_links,
        &static_links,
    );
    query.execute(&mut *ctx).await?;

    set_chall_updated(ctx, id).await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct NewChallInput {
    pub id: Option<Uuid>,
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

    pub flag: String,
}

pub async fn create_chall(ctx: &mut Ctx, input: NewChallInput) -> Result<Chall, sqlx::Error> {
    let query = query!(
        r#"
            INSERT INTO challenges (
                id,
                name, description, points,
                authors, hints, categories, tags,
                visible, source_folder, flag
            )
            VALUES (COALESCE($1, uuid_generate_v4()), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (source_folder)
            DO UPDATE SET
                id = COALESCE($1, uuid_generate_v4()),
                name = $2,
                description = $3,
                points = $4,
                authors = $5,
                hints = $6,
                categories = $7,
                tags = $8,
                visible = $9,
                source_folder = $10,
                flag = $11;
        "#,
        input.id,
        input.name: String,
        input.description,
        input.points,
        &input.authors,
        &input.hints,
        &input.categories,
        &input.tags,
        input.visible,
        &input.source_folder,
        input.flag
    );
    query.execute(&mut *ctx).await?;

    let Some(output) = get_chall_by_source_folder(&mut *ctx, &input.source_folder).await? else {
        return Err(sqlx::Error::RowNotFound);
    };

    set_chall_links(&mut *ctx, output.id, input.links).await?;

    set_chall_updated(ctx, output.id).await?;
    Ok(output)
}
