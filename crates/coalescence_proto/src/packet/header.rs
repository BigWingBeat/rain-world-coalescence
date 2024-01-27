use std::io::Write;

/// A header that prefixes all packets that are sent over the reliable-ordered channel, for framing
#[derive(Debug)]
pub struct OrderedHeader {
    /// The length of the serialized packet data
    pub length: usize,
}

impl OrderedHeader {
    pub const ENCODED_LEN: usize = std::mem::size_of::<u16>();

    /// Decode a header from an array
    pub fn decode(from: [u8; Self::ENCODED_LEN]) -> Self {
        Self {
            length: u16::from_le_bytes(from) as usize,
        }
    }

    /// Decode a header from a slice
    ///
    ///  # Panics
    ///
    /// Panics if the length of the slice is not equal to [`Self::ENCODED_LEN`]
    pub fn decode_from_slice(from: &[u8]) -> Self {
        let &[length_0, length_1] = from else {
            panic!(
                "OrderedHeader needs exactly {} bytes to decode, but {} bytes were given instead",
                Self::ENCODED_LEN,
                from.len()
            );
        };

        Self::decode([length_0, length_1])
    }

    pub fn encode_into<W: Write>(&self, mut into: W) -> std::io::Result<()> {
        into.write_all(&(self.length as u16).to_le_bytes())
    }
}
