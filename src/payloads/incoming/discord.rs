use std::fmt::Display;
use {
    serde::{Deserialize, Serialize},
    schemars::JsonSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AlertLevel {
    #[serde(rename = "INFO")]
    Info,
    #[serde(rename = "WARN")]
    Warn,
    #[serde(rename = "ERROR")]
    Erro,
}
impl Display for AlertLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Self::Erro => "Error",
            Self::Warn => "Warn ",
            Self::Info => "Info ",
        };
        f.write_str(val)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeveloperDiscordMessage {
    pub (crate) level: AlertLevel,
    pub (crate) message: String,
    pub (crate) data: serde_json::Value,
    pub (crate) include_chall_writers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__participant_type", rename_all = "snake_case")]
pub enum ParticipantMessage {
    FirstBlood { chall_name: String, team: String, user: String },
    Alert { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "__type", rename_all = "snake_case")]
pub enum ToDiscord {
    Developer(DeveloperDiscordMessage),
    Participant(ParticipantMessage),
}
