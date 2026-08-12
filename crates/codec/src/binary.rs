use serde::{Serialize, de::DeserializeOwned};

use crate::Codec;

#[derive(Debug, Clone, Copy)]
pub struct BinaryCodec;

impl Codec for BinaryCodec {
    type Error = BinaryCodecError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(value), err(Debug))
    )]
    fn encode<V: Serialize>(value: &V) -> Result<Vec<u8>, Self::Error> {
        postcard::to_allocvec(value).map_err(BinaryCodecError::Encode)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(raw), err(Debug))
    )]
    fn decode<V: DeserializeOwned>(raw: &[u8]) -> Result<V, Self::Error> {
        postcard::from_bytes(raw).map_err(BinaryCodecError::Decode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BinaryCodecError {
    #[error("encode: {0}")]
    Encode(postcard::Error),

    #[error("decode: {0}")]
    Decode(postcard::Error),
}
