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

pub async fn connection() -> Result<PoolConnection<Postgres>, Error> {
    static CONNECTION: OnceCell<PgPool> = OnceCell::const_new();

    let mutex = CONNECTION
        .get_or_init(|| async {

            let connection_options = PgConnectOptions::new()
                .application_name("ARCS-webhook")
                .database(cfg::db_name())
                .username(cfg::username());

            let connection_options: PgConnectOptions = if let Ok(password) = std::env::var("SQL_DB_PASS") {
                connection_options.password(&password)
            } else {
                connection_options
            };


            let options = PgPoolOptions::new()
                .min_connections(4)
                .max_connections(8);

            options
                .connect_with(connection_options)
                .await
                .unwrap()
        })
        .await;

    mutex
        .acquire()
        .await
}

pub async fn start_db_connection() {
    drop(connection().await);
}