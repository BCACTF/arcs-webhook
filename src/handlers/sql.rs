use async_trait::async_trait;

use crate::payloads::incoming::ToSql;
use crate::payloads::outgoing::sql::{FromSql, FromSqlErr};

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToSql {
    type SuccessPayload = FromSql;
    type ErrorPayload = FromSqlErr;
    async fn handle(self) -> ResponseFrom<Self> {
        todo!();
    }
}

