use serde::{Serialize, Deserialize};

use crate::handlers::OutgoingErr;

// TODO: Figure this out
#[derive(Debug, Clone, Serialize)]
#[serde(into = "&'static str")]
pub struct FromDiscord;

impl From<FromDiscord> for &'static str {
    fn from(_: FromDiscord) -> Self { "success" }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromDiscordErr {
    pub (crate) status_code: u16,
    pub (crate) status_message: String,
    pub (crate) body: Vec<u8>
}


impl OutgoingErr for FromDiscordErr {
    fn body(self) -> Result<serde_json::Value, String> {
        let message = "failed to send discord message";
        let code = self.status_code;
        let status_message = self.status_message;
        let body = String::from_utf8_lossy(&self.body).into_owned();

        Ok(serde_json::json!({
            "message": message,
            "code": code,
            "status_message": status_message,
            "body": body,
        }))
    }
    fn status_code(&self) -> u16 {
        self.status_code
    }
}
