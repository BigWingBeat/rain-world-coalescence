use serde::{Deserialize, Serialize};

#[cfg(not(feature = "bincode"))]
compile_error!(
    "Exactly one serde library needs to be enabled via a cargo feature, for example bincode"
);

#[cfg(feature = "bincode")]
pub type SerdeError = bincode::Error;

pub fn serialize<T>(value: &T) -> Result<Vec<u8>, SerdeError>
where
    T: Serialize + ?Sized,
{
    #[cfg(feature = "bincode")]
    bincode::serialize(value)
}

pub fn deserialize<'de, T>(bytes: &'de [u8]) -> Result<T, SerdeError>
where
    T: Deserialize<'de>,
{
    #[cfg(feature = "bincode")]
    bincode::deserialize(bytes)
}
