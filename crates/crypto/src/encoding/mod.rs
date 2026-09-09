#[cfg(feature = "base64")]
pub use base64::*;
#[cfg(feature = "hex")]
pub use hex::*;

#[cfg(feature = "base64")]
mod base64;
#[cfg(feature = "hex")]
mod hex;

pub trait EncodeBytes: AsRef<[u8]> {
    #[cfg(feature = "hex")]
    fn to_hex(&self) -> String {
        hex_encode(self.as_ref())
    }

    #[cfg(feature = "base64")]
    fn to_base64(&self) -> String {
        base64_encode(self.as_ref())
    }

    #[cfg(feature = "base64")]
    fn to_base64_url(&self) -> String {
        base64_url_encode(self.as_ref())
    }
}

impl<T: AsRef<[u8]>> EncodeBytes for T {}
