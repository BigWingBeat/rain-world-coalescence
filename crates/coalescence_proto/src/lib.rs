mod connection;
mod packet;
mod peer;
mod serde;
mod state;

pub use connection::Connection;
pub use peer::Peer;

use serde::CodecError;
use state::ConnectionState;
use strum::EnumCount;
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(ErrorKind);

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
    #[error(
        "Expected the connection to be in state {expected}, but it was in state {actual} instead"
    )]
    WrongState {
        expected: ConnectionState,
        actual: ConnectionState,
    },
    #[error(
		"{0} is not a valid connection state. Valid connection states range from 0 to {}",
		ConnectionState::COUNT - 1
	)]
    InvalidState(u8),
    #[error(transparent)]
    Serde(#[from] CodecError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
