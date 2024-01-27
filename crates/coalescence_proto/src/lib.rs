pub mod channel;
mod is;
pub mod packet;
pub mod peer;
mod plugin;
pub mod serde;

pub use is::Is;
pub use packet::{PacketReceiver, PacketSender, ReceiveError};
pub use plugin::{ConnectionBundle, ProtoPlugin, ReceivePackets, SendPackets};
pub use serde::ByteQueue;

use serde::SerdeError;
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(ErrorKind);

impl Error {
    pub(crate) fn kind(&self) -> &ErrorKind {
        &self.0
    }
}

impl<T> From<T> for Error
where
    T: Into<ErrorKind>,
{
    fn from(error: T) -> Self {
        Self(error.into())
    }
}

#[derive(Debug, Error)]
pub(crate) enum ErrorKind {
    #[error(transparent)]
    Serde(#[from] SerdeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
