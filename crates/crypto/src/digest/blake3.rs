use super::{Digest, Hash};

pub struct Blake3;

impl Hash for Blake3 {
    type Output = Digest;

    fn hash(message: &[u8]) -> Self::Output {
        Digest(blake3::hash(message).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_returns_a_32_byte_digest() {
        assert_eq!(Blake3::hash(b"a message").len(), 32);
    }

    #[test]
    fn hash_is_repeatable_for_the_same_message() {
        assert_eq!(Blake3::hash(b"a message"), Blake3::hash(b"a message"));
    }
}
