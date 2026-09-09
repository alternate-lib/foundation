const ALPHANUMERIC_CHARSET: &[u8; 62] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

pub fn numeric(len: usize) -> Vec<u8> {
    (0..len)
        .map(|_| rand::random_range::<u8, _>(b'0'..=b'9'))
        .collect()
}

pub fn numeric_array<const N: usize>() -> [u8; N] {
    std::array::from_fn(|_| rand::random_range(b'0'..=b'9'))
}

/// # Panics
///
/// Will panic if the generated numeric bytes are not valid UTF-8
pub fn numeric_string(len: usize) -> String {
    String::from_utf8(numeric(len)).expect("numeric generates only valid UTF-8")
}

pub fn alphanumeric(len: usize) -> Vec<u8> {
    (0..len)
        .map(|_| ALPHANUMERIC_CHARSET[rand::random_range(0..ALPHANUMERIC_CHARSET.len())])
        .collect()
}

pub fn alphanumeric_array<const N: usize>() -> [u8; N] {
    std::array::from_fn(|_| ALPHANUMERIC_CHARSET[rand::random_range(0..ALPHANUMERIC_CHARSET.len())])
}

/// # Panics
///
/// Will panic if the generated alphanumeric bytes are not valid UTF-8
pub fn alphanumeric_string(len: usize) -> String {
    String::from_utf8(alphanumeric(len)).expect("alphanumeric generates only valid UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numeric_returns_requested_length_and_digits() {
        let value = numeric(32);

        assert_eq!(value.len(), 32);
        assert!(value.iter().all(u8::is_ascii_digit));
    }

    #[test]
    fn numeric_array_returns_requested_length_and_digits() {
        let value = numeric_array::<32>();

        assert_eq!(value.len(), 32);
        assert!(value.iter().all(u8::is_ascii_digit));
    }

    #[test]
    fn alphanumeric_returns_requested_length_and_allowed_characters() {
        let value = alphanumeric(32);

        assert_eq!(value.len(), 32);
        assert!(
            value
                .iter()
                .all(|character| ALPHANUMERIC_CHARSET.contains(character))
        );
    }

    #[test]
    fn alphanumeric_array_returns_requested_length_and_allowed_characters() {
        let value = alphanumeric_array::<32>();

        assert_eq!(value.len(), 32);
        assert!(
            value
                .iter()
                .all(|character| ALPHANUMERIC_CHARSET.contains(character))
        );
    }
}
