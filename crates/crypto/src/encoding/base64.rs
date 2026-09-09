use base64::{Engine as _, engine::general_purpose};

pub fn base64_decode(encoded: &str) -> Result<Vec<u8>, Base64Error> {
    general_purpose::STANDARD
        .decode(encoded)
        .map_err(Base64Error::Decode)
}

pub fn base64_encode(data: &[u8]) -> String {
    general_purpose::STANDARD.encode(data)
}

pub fn base64_url_decode(encoded: &str) -> Result<Vec<u8>, Base64Error> {
    general_purpose::URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(Base64Error::Decode)
}

pub fn base64_url_encode(data: &[u8]) -> String {
    general_purpose::URL_SAFE_NO_PAD.encode(data)
}

#[derive(Debug, thiserror::Error)]
pub enum Base64Error {
    #[error("decode {0}")]
    Decode(#[from] base64::DecodeError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_binary_data() {
        let data = [0, 1, 2, 127, 128, 254, 255];

        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).unwrap();

        assert_ne!(encoded, "");
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_decode_wraps_decode_error() {
        let err = base64_decode("not valid base64").unwrap_err();

        assert!(matches!(err, Base64Error::Decode(_)));
    }

    #[test]
    fn base64_url_round_trips_binary_data_without_padding() {
        let data = [0, 1, 2, 127, 128, 254, 255];

        let encoded = base64_url_encode(&data);
        let decoded = base64_url_decode(&encoded).unwrap();

        assert_ne!(encoded, "");
        assert!(!encoded.ends_with('='));
        assert_eq!(decoded, data);
    }

    #[test]
    fn base64_url_decode_wraps_decode_error() {
        let err = base64_url_decode("not valid base64!").unwrap_err();

        assert!(matches!(err, Base64Error::Decode(_)));
    }
}
