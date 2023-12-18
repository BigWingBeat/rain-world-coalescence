use std::io::{Read, Write};

use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::SerdeCodec;

#[derive(Debug)]
pub struct Bincode;

impl Bincode {
    /// https://docs.rs/bincode/latest/bincode/config/index.html#options-struct-vs-bincode-functions
    pub fn options() -> impl bincode::Options {
        bincode::options()
    }
}

impl SerdeCodec for Bincode {
    type Error = bincode::Error;

    fn encode<T: Serialize + ?Sized>(data: &T) -> Result<Vec<u8>, Self::Error> {
        Self::options().serialize(data)
    }

    fn encode_into<T: Serialize + ?Sized, W: Write>(into: W, data: &T) -> Result<(), Self::Error> {
        Self::options().serialize_into(into, data)
    }

    fn encoded_size<T: Serialize + ?Sized>(data: &T) -> Result<usize, Self::Error> {
        Self::options().serialized_size(data).map(|size| {
            size.try_into()
                .expect("Size is too big to be represented as a usize")
        })
    }

    fn decode<'de, T: Deserialize<'de>>(bytes: &'de [u8]) -> Result<T, Self::Error> {
        Self::options().deserialize(bytes)
    }

    fn decode_from<T: DeserializeOwned, R: Read>(from: R) -> Result<T, Self::Error> {
        Self::options().deserialize_from(from)
    }
}
