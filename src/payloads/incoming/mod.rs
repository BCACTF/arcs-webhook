//! Defines the shape for an incoming message.
//! 
//! 
//! 
//! For subqueries, see:
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

/// This is the main struct that basically the entire server is built around.
/// 
/// It implements [`Handle`][crate::handlers::Handle] to make it easy to process
/// & respond to requests.
/// 
/// It's built to be as flexible as possible, but there are still some things
/// that are need to be done— notably allowing batch queries for each of the targets.
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct Incoming {
    /// Deploy query (create, poll, and stop deployments)
    #[serde(rename = "deploy")]
    pub (crate) depl: Option<ToDeploy>,

    /// Discord query (first blood and organizer notifications)
    #[serde(rename = "discord")]
    pub (crate) disc: Option<ToDiscord>,

    /// Frontend query (Resync cache)
    #[serde(rename = "frontend")]
    pub (crate) fron: Option<ToFrontend>,
    
    /// SQL query (addition, modification, and deletion of solves, teams, users, challenges, etc.)
    #[serde(rename = "sql")]
    pub (crate) sqll: Option<ToSql>,
}
