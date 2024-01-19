//! Packets are serialized with a length prefix for [framing], followed by the connection state, then the serialized packet data.
//! The packet data is encoded using the selected codec, but the length prefix is encoded manually, as a little-endian u16, and
//! the connection state is a single byte, and therefore doesn't require any encoding.
//!
//! The length prefix is encoded manually, rather than being put through the codec, because we need to know ahead-of-time
//! exactly how many bytes are needed to decode it. The codec would prevent us from knowing that, as it is a black box that
//! could encode integers in any way, including with a variable-length encoding such as zig-zag, elias gamma, etc.
//!
//! A u16 is used for the length prefix, because it's the smallest integer type that is guranteed to be able to store the length
//! of a packet without overflowing. The maximum allowed packet size varies per network path, but is [standardized] to always be
//! between 1,200 and 65,527 bytes, and the maximum value for a u16 is 65,535 bytes.
//!
//! [framing]: https://github.com/bincode-org/bincode/issues/519#issuecomment-1061925868
//! [standardized]: https://docs.rs/quinn-proto/0.10.6/quinn_proto/struct.MtuDiscoveryConfig.html#mtu-discovery-internals

use std::io::{Read, Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{state::ConnectionState, Error};

pub use byte_queue::ByteQueue;

mod byte_queue;

#[cfg(not(feature = "bincode"))]
compile_error!(
    "Exactly one serde library needs to be enabled via a cargo feature, for example bincode"
);

#[cfg(feature = "bincode")]
mod bincode;
#[cfg(feature = "bincode")]
use bincode::Bincode as Codec;

pub(crate) type CodecError = <Codec as SerdeCodec>::Error;

pub(crate) trait SerdeCodec {
    type Error: std::error::Error;

    fn encode<T: Serialize + ?Sized>(data: &T) -> Result<Vec<u8>, Self::Error>;
    fn encode_into<T: Serialize + ?Sized, W: Write>(into: W, data: &T) -> Result<(), Self::Error>;
    fn encoded_size<T: Serialize + ?Sized>(data: &T) -> Result<usize, Self::Error>;
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Self::Error>;
    fn decode_from<T: DeserializeOwned, R: Read>(from: R) -> Result<T, Self::Error>;
}

/// A header that prefixes all packets
#[derive(Debug)]
pub(crate) struct Header {
    /// The length of the serialized packet data
    pub(crate) length: usize,
    /// The packet's [`ConnectionState`]
    pub(crate) state: ConnectionState,
}

/// A [`Header`] that had an invalid [`ConnectionState`] value
#[derive(Debug)]
pub(crate) struct MalformedHeader {
    /// The length of the serialized packet data
    pub(crate) length: usize,
    /// The error that was produced when attempting to decode the invalid state value
    pub(crate) error: Error,
}

impl Header {
    pub const ENCODED_LEN: usize = 3;

    pub fn new<T>(state: ConnectionState, packet: &T) -> Result<Self, Error>
    where
        T: Serialize + ?Sized,
    {
        serialized_size(packet).map(|length| Self { length, state })
    }

    pub fn new_from_length(length: usize, state: ConnectionState) -> Self {
        Self { length, state }
    }

    pub fn decode(from: [u8; Self::ENCODED_LEN]) -> Result<Self, MalformedHeader> {
        let [length @ .., state] = from;
        let length = u16::from_le_bytes(length) as usize;
        state
            .try_into()
            .map(|state| Self { length, state })
            .map_err(|error| MalformedHeader { length, error })
    }

    // pub fn decode_from<R: Read>(mut from: R) -> Result<Self, MalformedHeader> {
    //     let mut buf = [0; Self::ENCODED_LEN];
    //     from.read_exact(&mut buf)
    //         .map_err(Into::into)
    //         .and_then(|_| Self::decode(buf))
    // }

    pub fn encode(&self) -> [u8; Self::ENCODED_LEN] {
        let [length_0, length_1] = (self.length as u16).to_le_bytes();
        [length_0, length_1, self.state.into()]
    }

    pub fn encode_to(&self, to: &mut [u8; Self::ENCODED_LEN]) {
        to.copy_from_slice(&self.encode());
    }

    pub fn encode_into<W: Write>(&self, mut into: W) -> Result<(), Error> {
        into.write_all(&self.encode()).map_err(Into::into)
    }
}

pub fn serialize<T>(data: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    Codec::encode(data).map_err(Into::into)
}

pub fn serialize_into<T, W: Write>(into: W, data: &T) -> Result<(), Error>
where
    T: Serialize + ?Sized,
{
    Codec::encode_into(into, data).map_err(Into::into)
}

pub fn serialized_size<T>(data: &T) -> Result<usize, Error>
where
    T: Serialize + ?Sized,
{
    Codec::encoded_size(data).map_err(Into::into)
}

pub fn deserialize_packet<'de, T>(bytes: &'de [u8]) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    Codec::decode(bytes).map_err(Into::into)
}

pub fn deserialize_packet_from<T, R: Read>(from: R) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    Codec::decode_from(from).map_err(Into::into)
}
