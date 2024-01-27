//! Packets are serialized with a length prefix for [framing], followed by the serialized packet data.
//! The packet data is encoded using the selected codec, but the length prefix is encoded manually, as a little-endian u16.
//!
//! The length prefix is encoded manually, rather than being put through the codec, because we need to know ahead-of-time
//! exactly how many bytes are needed to decode it. The codec would prevent us from knowing that, as it is a black box that
//! could encode integers in any way, including with a variable-length encoding such as zig-zag, elias gamma, etc.
//!
//! A u16 is used for the length prefix, because it's the smallest integer type that is guaranteed to be able to store the length
//! of a packet without overflowing. The maximum allowed packet size varies per network path, but is [standardized] to always be
//! between 1,200 and 65,527 bytes, and the maximum value for a u16 is 65,535 bytes.
//!
//! [framing]: https://github.com/bincode-org/bincode/issues/519#issuecomment-1061925868
//! [standardized]: https://docs.rs/quinn-proto/0.10.6/quinn_proto/struct.MtuDiscoveryConfig.html#mtu-discovery-internals

use std::io::{Read, Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{Error, ErrorKind};

mod byte_queue;

pub use byte_queue::{ByteQueue, Peek};

#[cfg(not(feature = "bincode"))]
compile_error!(
    "Exactly one serde library needs to be enabled via a cargo feature, for example bincode"
);

#[cfg(feature = "bincode")]
mod bincode;
#[cfg(feature = "bincode")]
use bincode::Bincode as Codec;

type CodecError = <Codec as SerdeCodec>::Error;

trait SerdeCodec {
    type Error: std::error::Error;

    fn encode<T: Serialize + ?Sized>(data: &T) -> Result<Vec<u8>, Self::Error>;
    fn encode_into<T: Serialize + ?Sized, W: Write>(into: W, data: &T) -> Result<(), Self::Error>;
    fn encoded_size<T: Serialize + ?Sized>(data: &T) -> Result<usize, Self::Error>;
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Self::Error>;
    fn decode_from<T: DeserializeOwned, R: Read>(from: R) -> Result<T, Self::Error>;
}

#[derive(Debug, Error)]
#[error(transparent)]
pub(crate) struct SerdeError(#[from] CodecError);

fn wrap_err(e: CodecError) -> Error {
    Error(ErrorKind::Serde(SerdeError(e)))
}

pub fn serialize<T>(data: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    Codec::encode(data).map_err(wrap_err)
}

pub fn serialize_into<T, W: Write>(into: W, data: &T) -> Result<(), Error>
where
    T: Serialize + ?Sized,
{
    Codec::encode_into(into, data).map_err(wrap_err)
}

pub fn serialized_size<T>(data: &T) -> Result<usize, Error>
where
    T: Serialize + ?Sized,
{
    Codec::encoded_size(data).map_err(wrap_err)
}

pub fn deserialize<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    Codec::decode(bytes).map_err(wrap_err)
}

pub fn deserialize_from<T, R: Read>(from: R) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    Codec::decode_from(from).map_err(wrap_err)
}
