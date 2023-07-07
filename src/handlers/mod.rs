pub mod deploy;
pub mod discord;
pub mod frontend;
pub mod sql;

use async_trait::async_trait;

use crate::payloads::{incoming::Incoming, outgoing::Outgoing};

/// This is a utility type getting the response type from a value implementing
/// `Handle`.
/// 
/// Here is an example in which it could be used:
/// ```
/// use async_trait::async_trait;
/// use webhook_rs::handlers::{ Handle, ResponseFrom };
/// 
/// struct EmptyPayload;
/// 
/// #[async_trait]
/// impl Handle for EmptyPayload {
///     type SuccessPayload = &'static str;
///     type ErrorPayload = std::convert::Infallible;
///     
///     // ResponseFrom<Self> is the same as Result<Self::SuccessPayload, Self::ErrorPayload> here
///     async fn handle(self) -> ResponseFrom<Self> {
///         Ok("It worked!")
///     }
/// }
/// ```
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

/// Implementing this trait allows the struct to be treated as data to be
/// executed. This is especially helpful for creating an JSON query input (as is
/// done for `webhook_rs`).
/// 
/// *One thing to note while implementing this type is that you will need to add
/// the `#[async_trait::async_trait]` attribute macro to allow for `handle`
/// being an `async fn`.*
#[async_trait]
pub trait Handle
where Self: Sized {
    /// The type the [`Handle::handle`] function will return if handled
    /// successfully.
    type SuccessPayload;
    /// The type the [`Handle::handle`] function will return if handled
    /// unsuccessfully or if there is an error in the handling.
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
