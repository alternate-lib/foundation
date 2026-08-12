use serde::{Serialize, de::DeserializeOwned};

use crate::Codec;

#[derive(Debug, Clone, Copy)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    type Error = JsonCodecError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(value), err(Debug))
    )]
    fn encode<V: Serialize>(value: &V) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(value).map_err(JsonCodecError::Encode)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(raw), err(Debug))
    )]
    fn decode<V: DeserializeOwned>(raw: &[u8]) -> Result<V, Self::Error> {
        serde_json::from_slice(raw).map_err(JsonCodecError::Decode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JsonCodecError {
    #[error("encode: {0}")]
    Encode(serde_json::Error),

    #[error("decode: {0}")]
    Decode(serde_json::Error),
}
