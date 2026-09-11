use super::{Digest, Hash, Hasher};

pub struct Sha256;

impl Hash for Sha256 {
    type Output = Digest;

    fn hash(message: &[u8]) -> Self::Output {
        Digest(<sha2::Sha256 as sha2::Digest>::digest(message).into())
    }
}

impl Sha256 {
    pub fn hasher() -> Sha256Hasher {
        Sha256Hasher::new()
    }
}

pub struct Sha256Hasher(sha2::Sha256);

impl Sha256Hasher {
    pub fn new() -> Self {
        Self(<sha2::Sha256 as sha2::Digest>::new())
    }
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Sha256Hasher {
    type Output = Digest;

    fn update(&mut self, message: &[u8]) -> &mut Self {
        <sha2::Sha256 as sha2::Digest>::update(&mut self.0, message);
        self
    }

    fn finalize(self) -> Self::Output {
        Digest(<sha2::Sha256 as sha2::Digest>::finalize(self.0).into())
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

    #[test]
    fn hasher_returns_a_32_byte_digest() {
        assert_eq!(Sha256::hasher().finalize().len(), 32);
    }

    #[test]
    fn hasher_matches_hash_when_updated_in_chunks() {
        let mut hasher = Sha256::hasher();
        hasher.update(b"a ").update(b"message");

        assert_eq!(hasher.finalize(), Sha256::hash(b"a message"));
    }
}
