//! Packets are serialized with a length prefix for [framing], followed by the connection state, then the serialized packet data.
//! The packet data is encoded using the selected codec, but the length prefix is encoded manually, as a little-endian u16, and
//! the connection state is a single byte.
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

#[cfg(not(feature = "bincode"))]
compile_error!(
    "Exactly one serde library needs to be enabled via a cargo feature, for example bincode"
);

#[cfg(feature = "bincode")]
mod bincode;
#[cfg(feature = "bincode")]
use bincode::Bincode as Codec;

pub(crate) type CodecError = <Codec as SerdeCodec>::Error;

use crate::{state::ConnectionState, Error, ErrorKind};

trait SerdeCodec {
    type Error: std::error::Error;

    fn encode<T: Serialize + ?Sized>(data: &T) -> Result<Vec<u8>, Self::Error>;
    fn encode_into<T: Serialize + ?Sized, W: Write>(into: W, data: &T) -> Result<(), Self::Error>;
    fn encoded_size<T: Serialize + ?Sized>(data: &T) -> Result<usize, Self::Error>;
    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Self::Error>;
    fn decode_from<T: DeserializeOwned, R: Read>(from: R) -> Result<T, Self::Error>;
}

pub const fn length_prefix_size() -> usize {
    std::mem::size_of::<u16>()
}

pub fn serialize<T>(data: &T, state: ConnectionState) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized,
{
    let encoded_size = Codec::encoded_size(data)?;
    let mut buffer = Vec::with_capacity(encoded_size + length_prefix_size() + 1);
    buffer.extend_from_slice(&(encoded_size as u16).to_le_bytes());
    buffer.push(state.into());
    Codec::encode_into(&mut buffer, data)?;
    buffer.shrink_to_fit();
    Ok(buffer)
}

pub fn serialize_into<T, W: Write>(
    mut into: W,
    data: &T,
    state: ConnectionState,
) -> Result<(), Error>
where
    T: Serialize + ?Sized,
{
    let encoded_size = Codec::encoded_size(data)? as u16;
    into.write_all(&encoded_size.to_le_bytes())?;
    into.write(&[state.into()])?;
    Codec::encode_into(into, data)?;
    Ok(())

    // serialize_impl(into, data, Codec::encoded_size(data)?).map_err(|e| Error(ErrorKind::Serde(e)))
}

pub fn serialized_size<T>(data: &T) -> Result<usize, Error>
where
    T: Serialize + ?Sized,
{
    Codec::encoded_size(data)
        .map(|size| size + length_prefix_size() + 1)
        .map_err(Into::into)
}

pub fn deserialize_length_prefix(bytes: [u8; 2]) -> usize {
    u16::from_le_bytes(bytes).into()
}

pub fn deserialize_connection_state(byte: u8) -> Result<ConnectionState, Error> {
    byte.try_into()
}

pub fn deserialize_packet<'de, T>(bytes: &'de [u8], state: ConnectionState) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    let [state_byte, bytes @ ..] = bytes else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Not enough bytes to deserialize slice",
        )
        .into());
    };

    let packet_state = (*state_byte).try_into()?;
    if state != packet_state {
        return Err(ErrorKind::WrongState {
            expected: state,
            actual: packet_state,
        }
        .into());
    }

    Codec::decode(bytes).map_err(Into::into)
}

pub fn deserialize_packet_from<T, R: Read>(mut from: R, state: ConnectionState) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let mut state_byte = [0];
    from.read_exact(&mut state_byte)?;
    let [state_byte] = state_byte;

    let packet_state = state_byte.try_into()?;
    if state != packet_state {
        return Err(ErrorKind::WrongState {
            expected: state,
            actual: packet_state,
        }
        .into());
    }

    Codec::decode_from(from).map_err(Into::into)
}
