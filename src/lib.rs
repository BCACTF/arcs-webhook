//! # ARCS Webhook
//! 
//! ### What is ARCS?
//! 
//! ARCS is a CTF framework designed by BCA's CTF Club— a group affiliated with
//! Bergen County Academies that runs an annual Capture the Flag cybersecurity
//! competition called `BCACTF`.
//! 
//! `BCACTF 4.0` – which took place in 2023 – is the first time an initial
//! prototype of the ARCS framework was used.
//! 
//! ARCS is based on a medium-sized-service architecture (mesoservices), and
//! there are 3 main custom servers built by the ARCS team.
//! 
//! The three servers are `frontend`, `deploy`, and `webhook`, with `webhook`
//! being the focus of this crate.
//! 
//! 
//! ## What does this crate even do?
//! 
//! This crate provides methods to handle webhook requests, along with 2
//! binaries, one for generating the incoming message schema, and another for
//! actually running the server.
//! 
//! Because the webhook is the "hub" of ARCS, there are 4 different targets
//! which it provides access to. These targets are:
//! 
//! - `frontend`
//! - `deploy`
//! - `sql`
//! - `discord`
//! 
//! The `frontend` and `deploy` targets are pretty self-explanitory, just
//! sending messages to the servers. The `sql` target has predefined queries,
//! creating a predefined set of actions to prevent sending raw SQL queries. The
//! `discord` target can send error/deploy messages of different types to the
//! CTF participants, the CTF admins, the CTF challenge writers, or any
//! combination of those. 
//! 
//! 
//! #### Something important to note:
//! 
//! _The webhook crate functions as the main "hub" of the system, and is
//! therefore a __SINGLE POINT OF FAILURE__. For this reason, it is written in
//! mostly safe rust, with a focus on __NEVER PANICKING OR CRASHING__._
//! 
//! ## Some things to note:
//! 
//! - [payloads::incoming::Incoming] is the shape of data sent to the webhook
//!   server.
//! - [payloads::outgoing::Outgoing] is the shape of data returned from the
//!   webhook server.
//! - The command `cargo run --bin generate_meta` will export the JSON schema
//!   for an incoming payload in `./meta/incoming.schema.json`.
//! 


#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
)]
#![warn(missing_docs)]

pub mod payloads;
pub mod handlers;

pub mod env;
mod auth;
mod sql;

pub use auth::{ AuthHeader, Token };
pub use sql::start_db_connection;

#[allow(unused_macros)]
pub mod logging {
    //! Contains the macros:
    //! 
    //! - [trace]
    //! - [debug]
    //! - [info]
    //! - [warn]
    //! - [error]
    //! 
    //! Each of these does correspond to a relevant

    use arcs_logging_rs::with_target;
    with_target! { "Webhook" }

    /// A display struct that helps with printing out user-entered information
    /// without having to worry about clogging up logs with escape sequences,
    /// long usernames, giant wrong flags, etc.
    /// 
    /// If the string is longer than the maximum number of characters, it is
    /// truncated and `...` is appended.
    /// 
    /// A shortened string can be created either by using [`Self::new()`] or
    /// [`shortened()`].
    pub struct Shortened<'a>(&'a str, bool);

    impl<'a> Shortened<'a> {
        /// Creates a new displayable shortened string. The lifetime of `string`
        /// determines the lifetime of the [Shortened].
        /// 
        /// `max_len` is the maximum length of the *raw characters* of the
        /// string. Please note that the length of the displayed string may be
        /// longer than the initial length of the string due to escaped control
        /// characters and control
        pub fn new(string: &'a str, max_len: usize) -> Self {
            let (display_name, shortened) =  if string.chars().count() >= max_len {
                if let Some((idx, _)) = string.char_indices().nth(max_len-3) {
                    (&string[..idx], true)
                } else { (string, false) }
            } else { (string, false) };
    
            Self(display_name, shortened)
        }
    }

    /// This is a utility function for the associated function
    /// [`Shortened::new()`][Shortened]. See that function for details.
    pub fn shortened(string: &str, max_len: usize) -> Shortened {
        Shortened::new(string, max_len)
    }

    impl<'a> std::fmt::Display for Shortened<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0.escape_debug())?;
            if self.1 {
                write!(f, "...")
            } else {
                Ok(())
            }
        }
    }
    impl<'a> std::fmt::Debug for Shortened<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "`{self}`")
        }
    }
}

mod http_client {
    use lazy_static::lazy_static;
    use reqwest::Client;

    lazy_static! {
        // FIXME: Think of a way to not use `unwrap`.
        #[warn(clippy::unwrap_used)]
        pub static ref DEFAULT: Client = {
            Client::builder()
                .user_agent("ARCS webhook requests")
                .build()
                .unwrap()
        };
    }
}
mod sql {
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
        pub fn str(&self) -> &str { &self.0 }
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
