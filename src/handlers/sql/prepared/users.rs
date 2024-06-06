use sqlx::{ query, query_as };
use uuid::Uuid;

use super::Ctx;
use crate::payloads::incoming::sql::Auth as CheckAuth;
use crate::payloads::outgoing::sql::{FromSqlErr, User};

pub async fn set_user_updated(ctx: &mut Ctx, id: Uuid) -> Result<u64, sqlx::Error> {
    let query = query!(
        r#"
            UPDATE users
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



pub async fn get_user(ctx: &mut Ctx, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let query = query_as!(
        User,
        r#"
            SELECT
                id, name as "name: _", email as "email: _",
                team_id, score, last_solve,
                admin, eligible
            FROM users WHERE id = $1;
        "#,
        id,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_user_by_name(ctx: &mut Ctx, name: &str) -> Result<Option<User>, sqlx::Error> {
    let query = query_as!(
        User,
        r#"
            SELECT
                id, name as "name: _", email as "email: _",
                team_id, score, last_solve,
                admin, eligible
            FROM users WHERE name = $1;
        "#,
        name: String,
    );
    query.fetch_optional(ctx).await
}

pub async fn get_all_users(ctx: &mut Ctx) -> Result<Vec<User>, sqlx::Error> {
    let query = query_as!(
        User,
        r#"
            SELECT
                id, name as "name: _", email as "email: _",
                team_id, score, last_solve,
                admin, eligible
            FROM users;
        "#,
    );
    query.fetch_all(ctx).await
}


#[derive(Debug, Clone)]
pub struct NewUserInput {
    pub name: String,
    pub email: String,
    pub eligible: bool,
    pub admin: bool,
    pub auth: Auth,
}


pub async fn create_user(ctx: &mut Ctx, input: NewUserInput) -> Result<User, FromSqlErr> {


    let query = query!(
        r#"
            INSERT INTO users (name, email, eligible, admin)
            VALUES ($1, $2, $3, $4);
        "#,
        input.name: String,
        input.email: String,
        input.eligible,
        input.admin,
    );
    query
        .execute(&mut *ctx)
        .await?;
    
    let user = get_user_by_name(&mut *ctx, &input.name).await?;
    let user = user.ok_or_else(|| sqlx::Error::RowNotFound)?;
    
    set_auth(&mut *ctx, user.id, input.auth).await?;
    set_user_updated(ctx, user.id).await?;

    Ok(user)
}

#[derive(Debug, Clone)]
pub enum Auth {
    OAuth { sub: String, provider: String, oauth_allow_token: String },
    Pass { hash: String },
}

pub async fn set_auth(ctx: &mut Ctx, id: Uuid, auth: Auth) -> Result<(), FromSqlErr> {

    let rows = match auth {
        Auth::OAuth { sub, provider, oauth_allow_token } => {
            use crate::auth::{ check_matches, Token::Oauth };
            if !check_matches(&[Oauth], oauth_allow_token.as_bytes()) {
                return Err(FromSqlErr::Auth);
            }

            query!(
                r#"
                    INSERT INTO auth_oauth ( user_id, sub, provider_name )
                    VALUES ($1, $2, $3);
                "#,
                id,
                sub,
                provider,
            ).execute(ctx).await?.rows_affected()
        },
        Auth::Pass { hash } => {
            query!(
                r#"
                    INSERT INTO auth_name_pass ( user_id, hashed_password )
                    VALUES ($1, $2);
                "#,
                id,
                hash,
            ).execute(ctx).await?.rows_affected()
        },
    };
    
    if rows != 1 { return Err(sqlx::Error::RowNotFound.into()) }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct UserInput {
    pub id: Uuid,
    pub name: Option<String>,
    pub eligible: Option<bool>,
    pub admin: Option<bool>,
}


pub async fn update_user(ctx: &mut Ctx, input: UserInput) -> Result<User, sqlx::Error> {

    let query = query!(
        r#"
            UPDATE users
            SET
                name = COALESCE($2, name),
                eligible = COALESCE($3, eligible),
                admin = COALESCE($4, admin)
            WHERE id = $1;
        "#,
        input.id,
        input.name: Option<String>,
        input.eligible,
        input.admin,
    );

    let affected = query
        .execute(&mut *ctx)
        .await?
        .rows_affected();

    set_user_updated(&mut *ctx, input.id).await?;

    if affected != 1 {
        return Err(sqlx::Error::RowNotFound)
    }


    if let Some(updated_user) = get_user(ctx, input.id).await? {
        Ok(updated_user)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}
pub async fn set_user_team(ctx: &mut Ctx, id: Uuid, team_id: Uuid, limit: Option<i64>) -> Result<User, sqlx::Error> {
    if let Some(limit) = limit {
        let team_can_take_more_users = query_as!(
            PredRow,
            r#"
                SELECT (COUNT(*) < $2) as "value!" FROM users WHERE team_id = $1;
            "#,
            team_id,
            limit,
        );
    
        let team_can_take_more_users = team_can_take_more_users.fetch_optional(&mut *ctx).await?;
        match team_can_take_more_users {
            Some(PredRow { value: false }) => return Err(sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Team is full."))),
            None => return Err(sqlx::Error::RowNotFound),
            Some(PredRow { value: true }) => {}
        }
    }

    let query = query!(
        r#"
            UPDATE users
            SET team_id = $2
            WHERE id = $1;
        "#,
        id,
        team_id
    );

    let affected = query
        .execute(&mut *ctx)
        .await?
        .rows_affected();

    set_user_updated(&mut *ctx, id).await?;

    if affected != 1 {
        return Err(sqlx::Error::RowNotFound)
    }

    if let Some(updated_user) = get_user(ctx, id).await? {
        Ok(updated_user)
    } else {
        Err(sqlx::Error::RowNotFound)
    }
}


pub enum CheckUserAuthError {
    Sql(sqlx::Error),
    Hashing,
    NotFound(Uuid),
    NotAllowed,
}

struct PasswordRow { hash: String }
struct CountRow { count: Option<i32> }

impl From<sqlx::Error> for CheckUserAuthError {
    fn from(value: sqlx::Error) -> Self {
        CheckUserAuthError::Sql(value)
    }
}
impl From<CheckUserAuthError> for FromSqlErr {
    fn from(value: CheckUserAuthError) -> Self {
        match value {
            CheckUserAuthError::Hashing => Self::OtherServerError("Failed to verify team password.".into()),
            CheckUserAuthError::Sql(_) => Self::DatabaseError,
            CheckUserAuthError::NotAllowed => Self::Auth,
            CheckUserAuthError::NotFound(id) => Self::DoesNotExist(id),
        }
    }
}

pub async fn check_user_auth(ctx: &mut Ctx, id: Uuid, auth: CheckAuth) -> Result<bool, CheckUserAuthError> {


    match auth {
        CheckAuth::OAuth { sub, provider, oauth_allow_token } => {
            use crate::auth::{ check_matches, Token::Oauth };
            if !check_matches(&[Oauth], oauth_allow_token.as_bytes()) {
                return Err(CheckUserAuthError::NotAllowed);
            }

            let query = query_as!(
                CountRow,
                r#"
                    SELECT COUNT(*)::integer FROM auth_oauth 
                    WHERE
                        user_id = $1 AND
                        sub = $2 AND
                        provider_name = $3;
                "#,
                id,
                sub,
                provider,
            );

            let result = query.fetch_optional(ctx).await?;

            if let Some(CountRow { count: Some(count) }) = result {
                Ok(count > 0)
            } else {
                Ok(false)
            }
        },
        CheckAuth::Pass { password } => {
            let query = query_as!(
                PasswordRow,
                r#"
                    SELECT hashed_password as hash FROM auth_name_pass WHERE id = $1;
                "#,
                id,
            );


            if let Some(PasswordRow { hash }) = query.fetch_optional(ctx).await? {
                Ok(
                    argon2::verify_encoded(
                        &hash,
                        password.as_bytes(),
                    ).map_err(|_| CheckUserAuthError::Hashing)?
                )
            } else {
                Err(CheckUserAuthError::NotFound(id))
            }
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserIsOnTeamOutcome { DoesNotExist, NotOnTeam, IsOnTeam }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PredRow { value: bool }

pub async fn user_is_on_team(ctx: &mut Ctx, id: Uuid, team_id: Uuid) -> Result<UserIsOnTeamOutcome, sqlx::Error> {

    let query = query_as!(
        PredRow,
        r#"
            SELECT (team_id = $2) as "value!" FROM users WHERE id = $1;
        "#,
        id,
        team_id,
    );

    let res = query.fetch_optional(ctx).await?;

    if let Some(row) = res {
        if row.value {
            Ok(UserIsOnTeamOutcome::IsOnTeam)
        } else {
            Ok(UserIsOnTeamOutcome::NotOnTeam)
        }
    } else {
        Ok(UserIsOnTeamOutcome::DoesNotExist)
    }
}

