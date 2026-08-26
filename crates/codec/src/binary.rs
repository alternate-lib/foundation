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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn round_trips_serializable_value() {
        let encoded = BinaryCodec::encode(&Example).unwrap();
        let decoded: Example = BinaryCodec::decode(&encoded).unwrap();

        assert_eq!(decoded, Example);
    }

    #[test]
    fn wraps_encode_failure() {
        let error = BinaryCodec::encode(&EncodeFailure).unwrap_err();

        assert!(matches!(error, BinaryCodecError::Encode(_)));
    }

    #[test]
    fn wraps_decode_failure() {
        let encoded = BinaryCodec::encode(&()).unwrap();
        let error = BinaryCodec::decode::<DecodeFailure>(&encoded).unwrap_err();

        assert!(matches!(error, BinaryCodecError::Decode(_)));
    }
}
