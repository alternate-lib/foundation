use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "binary")]
pub mod binary;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "msgpack")]
pub mod msgpack;

pub trait Codec {
    type Error: std::error::Error;

    fn encode<V: Serialize>(value: &V) -> Result<Vec<u8>, Self::Error>;

    fn decode<V: DeserializeOwned>(bytes: &[u8]) -> Result<V, Self::Error>;
}
