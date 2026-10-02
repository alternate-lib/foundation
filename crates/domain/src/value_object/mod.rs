pub use as_view::AsView;
#[cfg(feature = "email-address")]
pub use email_address::EmailAddress;
#[cfg(feature = "url")]
pub use url::Url;

pub mod as_view;
#[cfg(feature = "email-address")]
mod email_address;
#[cfg(feature = "url")]
mod url;

pub trait ValueObject: AsView + PartialEq + Sized {
    type Raw;
    type Error: std::error::Error;

    fn try_new(raw: Self::Raw) -> Result<Self, Self::Error>;

    fn into_raw(self) -> Self::Raw;
}
