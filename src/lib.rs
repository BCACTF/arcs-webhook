#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
)]

pub mod payloads;
pub mod handlers;

pub mod env;
mod auth;
mod sql;

pub use auth::{ AuthHeader, check_matches, Token };
pub use sql::start_db_connection;

#[allow(unused_macros)]
pub mod logging {
    use arcs_logging_rs::with_target;
    with_target! { "Webhook" }

    pub struct Shortened<'a>(&'a str, bool);
    pub fn shortened(string: &str, max_len: usize) -> Shortened {
        let (display_name, shortened) =  if string.chars().count() >= max_len {
            if let Some((idx, _)) = string.char_indices().nth(max_len-3) {
                (&string[..idx], true)
            } else { (string, false) }
        } else { (string, false) };

        Shortened(display_name, shortened)
    }

    impl<'a> std::fmt::Display for Shortened<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)?;
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
