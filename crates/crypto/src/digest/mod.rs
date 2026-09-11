use std::ops::Deref;

#[cfg(feature = "blake3")]
pub use blake3::*;
#[cfg(feature = "sha256")]
pub use sha256::*;

#[cfg(feature = "blake3")]
mod blake3;
#[cfg(feature = "sha256")]
mod sha256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digest([u8; 32]);

impl Digest {
    #[cfg(feature = "hex")]
    pub fn to_hex(&self) -> String {
        crate::EncodeBytes::to_hex(&self)
    }

    #[cfg(feature = "base64")]
    pub fn to_base64(&self) -> String {
        crate::EncodeBytes::to_base64(&self)
    }

    #[cfg(feature = "base64")]
    pub fn to_base64_url(&self) -> String {
        crate::EncodeBytes::to_base64_url(&self)
    }
}

impl AsRef<[u8]> for Digest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Deref for Digest {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Hash {
    type Output: AsRef<[u8]>;

    fn hash(message: &[u8]) -> Self::Output;
}

pub trait Hasher {
    type Output: AsRef<[u8]>;

    fn update(&mut self, message: &[u8]) -> &mut Self;

    fn finalize(self) -> Self::Output;
}
