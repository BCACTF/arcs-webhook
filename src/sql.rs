//! Contains types, setup functions, and connection handles to the postgresql
//! server all of the framework's data is stored in.

use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};

use sqlx::{
    PgPool,
    pool::PoolConnection,
    Postgres,
};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Error,
};

use tokio::sync::OnceCell;

use crate::env::sql as cfg;

#[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[sqlx(type_name = "citext")]
pub struct CiText(String);
impl CiText {
    pub fn string(self) -> String { self.0 }
    pub fn wrap(s: String) -> Self { Self(s) }
}


static CONNECTION: OnceCell<PgPool> = OnceCell::const_new();

pub async fn connection() -> Result<PoolConnection<Postgres>, Error> {
    CONNECTION
        .get()
        .ok_or(Error::PoolClosed)?
        .acquire()
        .await
}

/// This function initializes the database pool and connects with the env var
/// credentials.
/// 
/// If an error is returned, no connection has been established, and the
/// function is safe to call again.
/// 
/// If a success is returned, the pool connection is established and this
/// function should NOT be called again.
/// However, if it is called multiple times, it SHOULD just be an expensive
/// no-op.
pub async fn start_db_connection() -> Result<(), sqlx::Error> {
    CONNECTION.get_or_try_init(|| async {
        let connection_options = PgConnectOptions::new()
            .application_name("ARCS-webhook")
            .database(cfg::db_name())
            .username(cfg::username());

        let connection_options: PgConnectOptions = if let Ok(password) = std::env::var("SQL_DB_PASS") {
            connection_options.password(&password)
        } else {
            connection_options
        };


        let pool_options = PgPoolOptions::new()
            .min_connections(4)
            .max_connections(8);

        pool_options
            .connect_with(connection_options)
            .await
    }).await?;
    Ok(())
}
