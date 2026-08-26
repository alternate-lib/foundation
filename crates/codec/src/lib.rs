use serde::{Serialize, de::DeserializeOwned};

#[cfg(feature = "binary")]
pub mod binary;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "msgpack")]
pub mod msgpack;

pub trait Codec {
    type Error: std::error::Error;

    fn encode<V: Serialize>(value: &V) -> Result<Vec<u8>, Self::Error>;

    fn decode<V: DeserializeOwned>(bytes: &[u8]) -> Result<V, Self::Error>;
}

#[cfg(test)]
mod tests {
    use serde::{
        Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError,
        ser::Error as SerError,
    };

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    pub struct Example;

    #[derive(Debug)]
    pub struct EncodeFailure;

    impl Serialize for EncodeFailure {
        fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom("intentional encode failure"))
        }
    }

    #[derive(Debug)]
    pub struct DecodeFailure;

    impl<'de> Deserialize<'de> for DecodeFailure {
        fn deserialize<D>(_: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            Err(D::Error::custom("intentional decode failure"))
        }
    }
}
