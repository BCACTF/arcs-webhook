pub mod deploy;
pub mod discord;
pub mod frontend;
pub mod sql;

use async_trait::async_trait;

use crate::payloads::{incoming::Incoming, outgoing::Outgoing};

pub type ResponseFrom<T> = Result<<T as Handle>::SuccessPayload, <T as Handle>::ErrorPayload>;

pub trait OutgoingErr
where Self: Sized {
    fn status_code(&self) -> u16;
    fn body(self) -> Result<serde_json::Value, String>;
}

impl OutgoingErr for std::convert::Infallible {
    fn body(self) -> Result<serde_json::Value, String> {
        unreachable!()
    }
    fn status_code(&self) -> u16 {
        unreachable!()
    }
}


#[async_trait]
pub trait Handle
where Self: Sized {
    type SuccessPayload;
    type ErrorPayload: OutgoingErr;

    async fn handle(self) -> Result<Self::SuccessPayload, Self::ErrorPayload>;
}

#[async_trait]
impl Handle for Incoming {
    type SuccessPayload = Outgoing;
    type ErrorPayload = std::convert::Infallible;
    async fn handle(self) -> Result<Outgoing, std::convert::Infallible> {
        use futures::future::OptionFuture;

        let depl: OptionFuture<_> = self.depl.map(Handle::handle).into();
        let disc: OptionFuture<_> = self.disc.map(Handle::handle).into();
        let fron: OptionFuture<_> = self.fron.map(Handle::handle).into();
        let sqll: OptionFuture<_> = self.sqll.map(Handle::handle).into();

        let (
            depl,
            disc,
            fron,
            sqll,
        ) = tokio::join!(
            depl,
            disc,
            fron,
            sqll
        );

        Ok(Outgoing {
            depl,
            disc,
            fron,
            sqll,
        })
    }
}
