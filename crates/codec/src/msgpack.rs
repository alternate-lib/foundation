use serde::{Serialize, de::DeserializeOwned};

use crate::Codec;

#[derive(Debug, Clone, Copy)]
pub struct MsgpackCodec;

impl Codec for MsgpackCodec {
    type Error = MsgpackCodecError;

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(value), err(Debug))
    )]
    fn encode<V: Serialize>(value: &V) -> Result<Vec<u8>, Self::Error> {
        rmp_serde::to_vec(value).map_err(MsgpackCodecError::Encode)
    }

    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(level = "debug", skip(raw), err(Debug))
    )]
    fn decode<V: DeserializeOwned>(raw: &[u8]) -> Result<V, Self::Error> {
        rmp_serde::from_slice(raw).map_err(MsgpackCodecError::Decode)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MsgpackCodecError {
    #[error("encode: {0}")]
    Encode(rmp_serde::encode::Error),

    #[error("decode: {0}")]
    Decode(rmp_serde::decode::Error),
}
