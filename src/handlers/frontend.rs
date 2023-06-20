use async_trait::async_trait;

use crate::payloads::incoming::{ToFrontend};
use crate::payloads::outgoing::frontend::{FromFrontend, FromFrontendErr};

use super::{Handle, ResponseFrom};

#[async_trait]
impl Handle for ToFrontend {
    type SuccessPayload = FromFrontend;
    type ErrorPayload = FromFrontendErr;
    async fn handle(self) -> ResponseFrom<Self> {
        todo!();
    }
}
