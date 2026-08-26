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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::*;

    #[test]
    fn round_trips_serializable_value() {
        let encoded = JsonCodec::encode(&Example).unwrap();
        let decoded: Example = JsonCodec::decode(&encoded).unwrap();

        assert_eq!(decoded, Example);
    }

    #[test]
    fn wraps_encode_failure() {
        let error = JsonCodec::encode(&EncodeFailure).unwrap_err();

        assert!(matches!(error, JsonCodecError::Encode(_)));
    }

    #[test]
    fn wraps_decode_failure() {
        let encoded = JsonCodec::encode(&()).unwrap();
        let error = JsonCodec::decode::<DecodeFailure>(&encoded).unwrap_err();

        assert!(matches!(error, JsonCodecError::Decode(_)));
    }
}
