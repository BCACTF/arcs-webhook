//! Defines the shape for an incoming message.
//! 
//! See:
//! - [`frontend`]
//! - [`deploy`]
//! - [`discord`]
//! - [`sql`]

pub mod discord;
pub mod deploy;
pub mod sql;
pub mod frontend;

use serde::Deserialize;
use schemars::JsonSchema;

pub use {
    discord::ToDiscord,
    deploy::ToDeploy,
    frontend::ToFrontend,
    sql::ToSql,
};

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Incoming {
    #[serde(rename = "deploy")]
    pub (crate) depl: Option<ToDeploy>,
    #[serde(rename = "discord")]
    pub (crate) disc: Option<ToDiscord>,
    #[serde(rename = "frontend")]
    pub (crate) fron: Option<ToFrontend>,
    #[serde(rename = "sql")]
    pub (crate) sqll: Option<ToSql>,
}
