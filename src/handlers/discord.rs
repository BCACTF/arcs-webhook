use async_trait::async_trait;

use crate::http_client::DEFAULT;
use crate::payloads::incoming::{
    ToDiscord,
    discord::ParticipantMessage,
};
use crate::payloads::outgoing::discord::{ FromDiscord, FromDiscordErr };

use crate::env::discord as disc_env;

use crate::logging::*;

use super::{Handle, ResponseFrom};

use std::borrow::Cow;
use std::concat;
use std::fmt::Display;


#[derive(Debug, Clone)]
pub struct PayloadDetails {
    url: Cow<'static, str>,
    username: Cow<'static, str>,
    message: String,
}

pub struct Pings(Vec<&'static str>);

impl Display for Pings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for role_id in self.0.iter() {
            write!(f, "<@&{role_id}> ")?;
        }
        Ok(())
    }
}

impl ToDiscord {
    fn get_payload_details(self) -> PayloadDetails {
        match self {
            ToDiscord::Developer(dev_message) => {
                debug!("Discord req is a developer req");

                let url = if dev_message.include_chall_writers {
                    disc_env::chall_writer_url()
                } else {
                    disc_env::admin_url()
                };
                let pings = if dev_message.include_chall_writers {
                    vec![disc_env::chall_writer_role(), disc_env::admin_role()]
                } else {
                    vec![disc_env::admin_role()]
                };

                let message = format!(
                    concat!(
                        "-------------------", '\n',
                        "# Urgency: {}", '\n',
                        "{}", '\n',
                        "{}",
                    ),
                    dev_message.level,
                    Pings(pings),
                    dev_message.message,
                );

                PayloadDetails {
                    url: url.into(),
                    username: "ARCS Alerts".into(),
                    message,
                }
            }
            ToDiscord::Participant(message) => {
                debug!("Discord req is a developer req");

                let username = std::env::var("DISCORD_BOT_NAME").ok().map(Into::into);
                let username = username.unwrap_or("CTF Updates".into());
                
                let url = disc_env::participant_url().into();

                let message = match message {
                    ParticipantMessage::Alert { message } => message,
                    ParticipantMessage::FirstBlood { chall_name, team, user } => {
                        let team = team.replace('`', "'");
                        let user = user.replace('`', "'");

                        format!("First :drop_of_blood: by `{user}` from `{team}` on challenge `{chall_name}`!")
                    }
                };
                PayloadDetails {
                    url,
                    username,
                    message,
                }
            }
        }
    }
}

#[async_trait]
impl Handle for ToDiscord {
    type SuccessPayload = FromDiscord;
    type ErrorPayload = FromDiscordErr;
    async fn handle(self) -> ResponseFrom<ToDiscord> {
        trace!("Handling discord webhook req");

        let PayloadDetails { url, username, message } = self.get_payload_details();

        let body = serde_json::json!({
            "username": username,
            "content": message,
        });

        let response = DEFAULT
            .post(&*url)
            .json(&body)
            .send()
            .await;


        match response {
            Ok(response) => if response.status().is_success() {
                info!("Discord webhook req successful");
                Ok(FromDiscord)
            } else {
                warn!("Discord webhook req failed");

                let status_code = response.status().as_u16();
                let status_message = response
                    .headers()
                    .get("Reason-Phrase")
                    .and_then(|v|  v.to_str().ok())
                    .map(str::to_string)
                    .or_else(|| response.status().canonical_reason().map(str::to_string))
                    .unwrap_or_default();

                let err = match response.bytes().await {
                    Ok(body) => FromDiscordErr {
                        status_code,
                        status_message,
                        body: body.to_vec(),
                    },
                    Err(_) => FromDiscordErr {
                        status_code: 500,
                        status_message: "Failed to read discord response".into(),
                        body: "".into(),
                    }
                };
                Err(err)
            },
            Err(_) => {
                error!("Sending request to discord failed. This could signal a major issue.");

                let err = FromDiscordErr {
                    status_code: 500,
                    status_message: "Failed to send request to discord".into(),
                    body: "".into(),
                };

                Err(err)
            },
        }
    }
}
