use crate::logging::*;
use crate::payloads::*;

use futures::TryFutureExt;
use incoming::sql::{ UserQuery, Auth as IncomingAuth };
use outgoing::sql::{FromSql, FromSqlErr};

use super::prepared::users as queries;
use queries::{
    get_all_users, get_user, get_user_by_name,
    create_user, update_user, set_user_team,
    check_user_auth, set_auth,
};
use queries::{ UserInput, NewUserInput, Auth as SqlAuth };

fn get_create_auth(auth: IncomingAuth) -> Result<SqlAuth, FromSqlErr> {
    use crate::passwords::*;

    let auth_val = match auth {
        IncomingAuth::OAuth { sub, provider, oauth_allow_token } => SqlAuth::OAuth { sub, provider, oauth_allow_token },
        IncomingAuth::Pass { password } => {
            let Ok(salt) = salt() else {
                return Err(FromSqlErr::OtherServerError("Failed to hash team password.".into()))
            };
            let Ok(hash) = argon2::hash_encoded(password.as_bytes(), &salt, &ARGON2_CONFIG) else {
                return Err(FromSqlErr::OtherServerError("Failed to hash team password.".into()))
            };
            SqlAuth::Pass { hash }
        }
    };
    Ok(auth_val)
}

pub async fn handle(query: UserQuery) -> Result<FromSql, FromSqlErr> {
    trace!("Handling SQL user req");

    let success_res = match query {
        UserQuery::GetAllUsers => {
            debug!("SQL user req classified as 'GetAllUsers' req");
            FromSql::UserArr(get_all_users().await?)
        },
        UserQuery::GetUser { id } => {
            debug!("SQL user req classified as 'GetUser<{id}>' req");

            if let Some(user) = get_user(id).await? {
                FromSql::User(user)
            } else {
                return Err(FromSqlErr::DoesNotExist(id))
            }
        },
        UserQuery::CheckUsernameAvailability { name } => {
            let display_name = shortened(&name, 13);
            debug!("SQL user req classified as 'CheckUsernameAvailability<`{display_name}`>' req");

            let user = get_user_by_name(&name).await?;
            FromSql::Availability(user.is_none())
        },
        UserQuery::CreateNewUser { name, email, eligible, admin, auth } => {
            let display_name = shortened(&name, 13);
            let elig_str = if eligible { "elig " } else { "" };
            let admin_str = if admin { "admin " } else { "" };
            debug!("SQL user req classified as 'CreateNewUser<`{display_name}` ( {elig_str}{admin_str})>' req");

            let auth = get_create_auth(auth)?;

            FromSql::User(
                create_user(NewUserInput {
                    name,
                    email,
                    eligible,
                    admin,
                    auth,
                }).await?
            )
        },
        UserQuery::UpdateUserAuth { id, old_auth, new_auth } => {
            debug!("SQL user req classified as 'UpdateUserAuth<{id}>' req");

            if !check_user_auth(id, old_auth).await? {
                return Err(FromSqlErr::Auth)
            }
            set_auth(id, get_create_auth(new_auth)?).await?;
            FromSql::User(get_user(id).await?.ok_or(sqlx::Error::RowNotFound)?)
        },
        UserQuery::CheckUserAuth { id, auth } => {
            debug!("SQL user req classified as 'CheckUserAuth<{id}>' req");
            FromSql::AuthStatus(check_user_auth(id, auth).await?)
        },
        UserQuery::JoinTeam { id, auth, team_name, team_pass } => {
            let display_name = shortened(&team_name, 13);
            debug!("SQL user req classified as 'JoinTeam<{id} joining {display_name}>' req");

            let Some(team) = super::prepared::teams::get_team_by_name(&team_name).await? else {
                return Err(FromSqlErr::NameDoesNotExist(team_name))
            };

            let user_auth = check_user_auth(id, auth).map_err(FromSqlErr::from);
            let team_auth = super::prepared::teams::check_team_auth(team.id, team_pass).map_err(FromSqlErr::from);
            let (user_auth, team_auth) = tokio::try_join!(user_auth, team_auth)?;
            
            if user_auth && team_auth {
                FromSql::User(set_user_team(id, team.id).await?)
            } else {
                return Err(FromSqlErr::Auth)
            }
        }
    };
    Ok(success_res)
}
