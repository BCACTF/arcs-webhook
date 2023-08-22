mod env {
    use arcs_env_rs::*;

    mod str_vars {
        use arcs_env_rs::*;

        env_var_req!(ALLOWED_OAUTH_TOKEN -> OAUTH_TOKEN);

        env_var_req!(FRONTEND_AUTH_TOKEN -> FRONTEND_TOKEN);
        env_var_req!(WEBHOOK_AUTH_TOKEN -> WEBHOOK_TOKEN);
        env_var_req!(DEPLOY_AUTH_TOKEN -> DEPLOY_TOKEN);

        assert_req_env!(check_str_env_vars:
            FRONTEND_TOKEN, WEBHOOK_TOKEN, DEPLOY_TOKEN,
            OAUTH_TOKEN
        );
    }
    mod len_vars {
        use super::str_vars::{ frontend_token, webhook_token, deploy_token, oauth_token };
        use arcs_env_rs::*;

        lazy_static::lazy_static! {
            pub static ref FRONTEND_AUTH: Result<[u8; 32], &'static str> = (&frontend_token().as_bytes().to_owned()[..])
                .try_into()
                .map_err(|_| "FRONTEND_AUTH");
            pub static ref WEBHOOK_AUTH:  Result<[u8; 32], &'static str> = (&webhook_token() .as_bytes().to_owned()[..])
                .try_into()
                .map_err(|_| "WEBHOOK_AUTH");
            pub static ref DEPLOY_AUTH:   Result<[u8; 32], &'static str> = (&deploy_token()  .as_bytes().to_owned()[..])
                .try_into()
                .map_err(|_| "DEPLOY_AUTH");
            pub static ref OAUTH_AUTH:    Result<[u8; 32], &'static str> = (&oauth_token()   .as_bytes().to_owned()[..])
                .try_into()
                .map_err(|_| "OAUTH_AUTH");
        }

        pub fn frontend_auth() -> [u8; 32] {
            FRONTEND_AUTH.unwrap()
        }
        pub fn webhook_auth() -> [u8; 32] {
            WEBHOOK_AUTH.unwrap()
        }
        pub fn deploy_auth() -> [u8; 32] {
            DEPLOY_AUTH.unwrap()
        }
        pub fn oauth_auth() -> [u8; 32] {
            OAUTH_AUTH.unwrap()
        }

        assert_req_env!(check_len_env_vars:
            FRONTEND_AUTH, WEBHOOK_AUTH, DEPLOY_AUTH,
            OAUTH_AUTH
        );
    }

    pub fn check_env_vars() -> Result<(), EnvVarErr<4>> {
        str_vars::check_str_env_vars()?;
        len_vars::check_len_env_vars()
    }

    pub use len_vars::{ frontend_auth, webhook_auth, deploy_auth, oauth_auth };
}
pub use env::{ webhook_auth, check_env_vars };

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Frontend,
    Deploy,
    Oauth,
}


pub fn check_matches(list: &[Token], bytes: &[u8]) -> bool {
    fn black_box_or(val_1: bool, val_2: bool) -> bool {
        val_1 || val_2
    }

    let Ok(buffer) = bytes.to_owned()[..].try_into() else { return false };

    let mut will_return_true = false;
    
    for token in list {
        use self::env::*;

        let bool_return = match token {
            Token::Frontend => constant_time_eq::constant_time_eq_32(&buffer, &frontend_auth()),
            Token::Deploy   => constant_time_eq::constant_time_eq_32(&buffer, &deploy_auth()),
            Token::Oauth    => constant_time_eq::constant_time_eq_32(&buffer, &oauth_auth()),
        };
        will_return_true = std::hint::black_box(black_box_or(
            std::hint::black_box(bool_return),
            std::hint::black_box(will_return_true),
        ));
    }
    will_return_true
}

// pub fn authenticate_request()


#[derive(Debug, Clone)]
pub struct AuthHeader {
    data: Vec<u8>,
}

use actix_web::{
    http::header::{
        TryIntoHeaderValue, AUTHORIZATION,
        Header
    },
    error::ParseError,
};

impl TryIntoHeaderValue for AuthHeader {
    type Error = <Vec<u8> as TryIntoHeaderValue>::Error;

    fn try_into_value(self) -> Result<reqwest::header::HeaderValue, Self::Error> {
        self.data.try_into_value()
    }
}
impl Header for AuthHeader {
    fn name() -> reqwest::header::HeaderName { AUTHORIZATION }

    fn parse<M: actix_web::HttpMessage>(msg: &M) -> Result<Self, actix_web::error::ParseError> {
        let header = msg
            .headers()
            .get(Self::name())
            .ok_or(ParseError::Header)?;

        Ok(Self { data: header.as_bytes().to_vec() })
    }
}

impl AuthHeader {
    pub fn check_matches(&self, list: &[Token]) -> bool {
        let Some(stripped) = self.data.strip_prefix(b"Bearer ") else { return false; };

        check_matches(list, stripped)
    }
}
