pub mod discord;
pub mod deploy;
pub mod sql;
pub mod frontend;

use serde::Deserialize;

pub use {
    discord::ToDiscord,
    deploy::ToDeploy,
    frontend::ToFrontend,
    sql::ToSql,
};

#[derive(Debug, Clone, Deserialize)]
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
