use crate::{
    AsView, ValueObject,
    validator::{self, string::UrlError},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Url(String);

impl Url {
    pub fn try_new(raw: impl Into<String>) -> Result<Self, UrlError> {
        let raw = raw.into();
        Self::validate(&raw)?;
        Ok(Self(raw))
    }

    pub fn validate(value: &str) -> Result<(), UrlError> {
        <Self as ValueObject>::validate(value)
    }

    pub fn as_view(&self) -> &str {
        <Self as AsView>::as_view(self)
    }

    pub fn as_str(&self) -> &str {
        <Self as AsView>::as_view(self)
    }

    pub fn into_raw(self) -> String {
        <Self as ValueObject>::into_raw(self)
    }
}

impl AsView for Url {
    type View = str;

    fn as_view(&self) -> &str {
        self.0.as_str()
    }
}

impl ValueObject for Url {
    type Raw = String;
    type Error = UrlError;

    fn validate(value: &str) -> Result<(), Self::Error> {
        validator::string::url(value)
    }

    fn into_raw(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Url {
    type Error = UrlError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_new(raw)
    }
}

impl AsRef<str> for Url {
    fn as_ref(&self) -> &str {
        <Self as AsView>::as_view(self)
    }
}
