pub fn hex_encode(input: &[u8]) -> String {
    hex::encode(input)
}

pub fn hex_decode(input: &str) -> Result<Vec<u8>, HexError> {
    hex::decode(input).map_err(HexError::Decode)
}

#[derive(Debug, thiserror::Error)]
pub enum HexError {
    #[error("decode hex: {0}")]
    Decode(hex::FromHexError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_binary_data() {
        let data = [0, 1, 2, 127, 128, 254, 255];

        let encoded = hex_encode(&data);
        let decoded = hex_decode(&encoded).unwrap();

        assert_eq!(encoded.len(), data.len() * 2);
        assert!(
            encoded
                .bytes()
                .all(|character| character.is_ascii_hexdigit())
        );
        assert_eq!(decoded, data);
    }

    #[test]
    fn hex_decode_wraps_decode_error() {
        let err = hex_decode("not hex").unwrap_err();

        assert!(matches!(err, HexError::Decode(_)));
    }
}
