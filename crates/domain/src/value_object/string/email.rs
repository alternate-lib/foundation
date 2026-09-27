use crate::{
    AsView, ValueObject,
    validator::{self, string::EmailError},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Email(String);

impl Email {
    pub fn try_new(raw: impl Into<String>) -> Result<Self, EmailError> {
        let raw = raw.into();
        Self::validate(&raw)?;

        Ok(Self(raw))
    }

    pub fn validate(value: &str) -> Result<(), EmailError> {
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

impl AsView for Email {
    type View = str;

    fn as_view(&self) -> &str {
        self.0.as_str()
    }
}

impl ValueObject for Email {
    type Raw = String;
    type Error = EmailError;

    fn validate(value: &str) -> Result<(), Self::Error> {
        validator::string::email(value)
    }

    fn into_raw(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Email {
    type Error = EmailError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        Self::try_new(raw)
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        <Self as AsView>::as_view(self)
    }
}
