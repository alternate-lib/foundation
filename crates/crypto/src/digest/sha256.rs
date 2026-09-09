use super::{Digest, Hash};

pub struct Sha256;

impl Hash for Sha256 {
    type Output = Digest;

    fn hash(message: &[u8]) -> Self::Output {
        Digest(<sha2::Sha256 as sha2::Digest>::digest(message).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_returns_a_32_byte_digest() {
        assert_eq!(Sha256::hash(b"a message").len(), 32);
    }

    #[test]
    fn hash_is_repeatable_for_the_same_message() {
        assert_eq!(Sha256::hash(b"a message"), Sha256::hash(b"a message"));
    }
}
