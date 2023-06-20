use serde::Serialize;

use crate::handlers::OutgoingErr;

// TODO: Fix this
#[derive(Debug, Clone, Serialize)]
pub struct FromSql {

}

#[derive(Debug, Clone, Serialize)]
pub struct FromSqlErr {

}

impl OutgoingErr for FromSqlErr {
    fn body(self) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!("Unimplemented"))
    }
    fn status_code(&self) -> u16 {
        500
    }
}
