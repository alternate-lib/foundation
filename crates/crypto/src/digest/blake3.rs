use super::{Digest, Hash, Hasher};

pub struct Blake3;

impl Hash for Blake3 {
    type Output = Digest;

    fn hash(message: &[u8]) -> Self::Output {
        Digest(blake3::hash(message).into())
    }
}

impl Blake3 {
    pub fn hasher() -> Blake3Hasher {
        Blake3Hasher::new()
    }
}

pub struct Blake3Hasher(blake3::Hasher);

impl Blake3Hasher {
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }
}

impl Default for Blake3Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Blake3Hasher {
    type Output = Digest;

    fn update(&mut self, message: &[u8]) -> &mut Self {
        self.0.update(message);
        self
    }

    fn finalize(self) -> Self::Output {
        Digest(self.0.finalize().into())
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

    #[test]
    fn hasher_returns_a_32_byte_digest() {
        assert_eq!(Blake3::hasher().finalize().len(), 32);
    }

    #[test]
    fn hasher_matches_hash_when_updated_in_chunks() {
        let mut hasher = Blake3::hasher();
        hasher.update(b"a ").update(b"message");

        assert_eq!(hasher.finalize(), Blake3::hash(b"a message"));
    }
}
