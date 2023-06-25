pub mod payloads;
pub mod handlers;

mod auth;

#[allow(unused_macros)]
pub mod logging {
    use arcs_logging_rs::with_target;
    with_target! { "Webhook" }
}


pub mod env {
    use arcs_env_rs::*;

    env_var_req!(PORT);

    env_var_req!(FRONTEND_ADDRESS);
    env_var_req!(WEBHOOK_ADDRESS);
    env_var_req!(DEPLOY_ADDRESS);
        
    assert_req_env!(check_env_vars:
        PORT,
        FRONTEND_ADDRESS, WEBHOOK_ADDRESS, DEPLOY_ADDRESS
    );

    pub mod discord {
        use arcs_env_rs::*;

        env_var_req!(DISCORD_ADMIN_WEBHOOK_URL -> ADMIN_URL);
        env_var_req!(DISCORD_CHALL_WRITER_WEBHOOK_URL -> CHALL_WRITER_URL);
        env_var_req!(DISCORD_PARTICIPANT_URL -> PARTICIPANT_URL);

        env_var_req!(DISCORD_ADMIN_ROLE_ID -> ADMIN_ROLE);
        env_var_req!(DISCORD_CHALL_WRITER_ROLE_ID -> CHALL_WRITER_ROLE);
        env_var_req!(DISCORD_PARTICIPANT_ROLE_ID -> PARTICIPANT_ROLE);

        assert_req_env!(
            check_env_vars:
                ADMIN_URL,  CHALL_WRITER_URL,  PARTICIPANT_URL,
                ADMIN_ROLE, CHALL_WRITER_ROLE, PARTICIPANT_ROLE
        );
    }

    pub mod sql {
        use arcs_env_rs::*;

        env_var_req!(SQL_DB_NAME -> DB_NAME);
        // env_var_req!(SQL_DB_PASS -> DB_PASS);

        env_var_req!(SQL_USERNAME -> USERNAME);

        assert_req_env!(
            check_env_vars:
                DB_NAME, // DB_PASS,
                USERNAME
        );
    }

    pub mod checks {
        pub use super::check_env_vars as main;
        pub use super::discord::check_env_vars as discord;
        pub use super::sql::check_env_vars as sql;
        pub use crate::auth::check_env_vars as auth;
    }
}



mod http_client {
    use lazy_static::lazy_static;
    use reqwest::Client;

    lazy_static! {
        pub static ref DEFAULT: Client = {
            Client::builder()
                .user_agent("ARCS webhook requests")
                .build()
                .unwrap()
        };
    }
}
mod sql {
    use serde::{Serialize, Deserialize};

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

    #[derive(sqlx::Type, Debug, Clone, Serialize, Deserialize)]
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
}
pub use sql::start_db_connection;

mod passwords {
    use argon2::{ Config, ThreadMode, Variant, Version };
    
    pub const ARGON2_CONFIG: Config = Config {
        mem_cost: 65536,
        time_cost: 11,
        lanes: 4,
        secret: &[],
        ad: &[],
        hash_length: 32,

        variant: Variant::Argon2i,
        version: Version::Version13,
        thread_mode: ThreadMode::Parallel,
    };
    
    use rand::rngs::StdRng;
    use rand::{SeedableRng, RngCore};
    use std::sync::Mutex;

    lazy_static::lazy_static! {
        static ref SALTER: Mutex<StdRng> = Mutex::new(StdRng::from_entropy());
    }
    pub fn salt() -> Result<[u8; 32], ()> {
        let mut salt = [0; 32];

        SALTER
            .try_lock().map_err(|_| ())?
            .try_fill_bytes(&mut salt)
            .map_err(|_| ())?;

        Ok(salt)
    }
}
