use crate::{ValidationErrors, Validator};

#[derive(Debug, Clone, Copy, Default)]
pub struct EmailAddress;

impl Validator<str> for EmailAddress {
    type Error = EmailAddressError;

    fn validate(&self, value: &str) -> Result<(), ValidationErrors<Self::Error>> {
        let _: email_address::EmailAddress = value
            .parse()
            .map_err(EmailAddressError::from)
            .map_err(ValidationErrors::single)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EmailAddressError {
    #[error("invalid character found")]
    InvalidCharacter,

    #[error("component missing: {0}")]
    Missing(&'static str),

    #[error("component empty: {0}")]
    Empty(&'static str),

    #[error("component too long: {0}")]
    TooLong(&'static str),

    #[error("component invalid: {0}")]
    Invalid(&'static str),

    #[error("{0}")]
    Other(String),
}

impl From<email_address::Error> for EmailAddressError {
    fn from(value: email_address::Error) -> Self {
        match value {
            email_address::Error::InvalidCharacter => Self::InvalidCharacter,
            email_address::Error::MissingSeparator => Self::Missing("separator"),
            email_address::Error::MissingDisplayName => Self::Missing("display name"),
            email_address::Error::MissingEndBracket => Self::Missing("end bracket"),
            email_address::Error::LocalPartEmpty => Self::Empty("local part"),
            email_address::Error::DomainEmpty => Self::Empty("domain"),
            email_address::Error::SubDomainEmpty => Self::Empty("subdomain"),
            email_address::Error::LocalPartTooLong => Self::TooLong("local part"),
            email_address::Error::DomainTooLong => Self::TooLong("domain"),
            email_address::Error::SubDomainTooLong => Self::TooLong("subdomain"),
            email_address::Error::DomainInvalidSeparator => Self::Invalid("domain separator"),
            email_address::Error::InvalidIPAddress => Self::Invalid("domain ip address"),
            _ => Self::Other(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_address_valid_inputs() {
        for value in [
            "user@example.com",
            "user+tag@example.com",
            "user@sub.example.com",
        ] {
            assert_eq!(EmailAddress.validate(value), Ok(()), "{value:?}");
        }
    }

    #[test]
    fn email_address_invalid_inputs() {
        for (value, error) in [
            ("user.example.com", EmailAddressError::Missing("separator")),
            ("@example.com", EmailAddressError::Empty("local part")),
            ("user@", EmailAddressError::Empty("domain")),
            ("user@example..com", EmailAddressError::Empty("subdomain")),
        ] {
            assert_eq!(
                EmailAddress.validate(value),
                Err(ValidationErrors::single(error)),
                "{value:?}"
            );
        }
        assert!(EmailAddress.validate("").is_err());
        assert_eq!(
            EmailAddress.validate(&format!("{}@example.com", "a".repeat(65))),
            Err(ValidationErrors::single(EmailAddressError::TooLong(
                "local part"
            )))
        );
    }
}
