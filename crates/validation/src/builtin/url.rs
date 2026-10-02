use crate::{ValidationErrors, Validator};

#[derive(Debug, Clone, Copy, Default)]
pub struct Url;

impl Validator<str> for Url {
    type Error = UrlError;

    fn validate(&self, value: &str) -> Result<(), ValidationErrors<Self::Error>> {
        let _: url::Url = value
            .parse()
            .map_err(UrlError::from)
            .map_err(ValidationErrors::single)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum UrlError {
    #[error("relative URL without a base")]
    RelativeUrlWithoutBase,

    #[error("component empty: {0}")]
    Empty(&'static str),

    #[error("component invalid: {0}")]
    Invalid(&'static str),

    #[error("{0}")]
    Other(String),
}

impl From<url::ParseError> for UrlError {
    fn from(value: url::ParseError) -> Self {
        match value {
            url::ParseError::RelativeUrlWithoutBase => Self::RelativeUrlWithoutBase,
            url::ParseError::EmptyHost => Self::Empty("host"),
            url::ParseError::InvalidPort => Self::Invalid("port"),
            url::ParseError::InvalidIpv4Address => Self::Invalid("ipv4 address"),
            url::ParseError::InvalidIpv6Address => Self::Invalid("ipv6 address"),
            url::ParseError::IdnaError | url::ParseError::InvalidDomainCharacter => {
                Self::Invalid("domain")
            }
            _ => Self::Other(value.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_valid_inputs() {
        for value in [
            "https://example.com",
            "https://example.com:8080/path?query=value#fragment",
            "http://127.0.0.1",
            "http://[::1]",
            "mailto:user@example.com",
        ] {
            assert_eq!(Url.validate(value), Ok(()), "{value:?}");
        }
    }

    #[test]
    fn url_invalid_inputs() {
        for (value, error) in [
            ("", UrlError::RelativeUrlWithoutBase),
            ("/path", UrlError::RelativeUrlWithoutBase),
            ("example.com", UrlError::RelativeUrlWithoutBase),
            ("https://", UrlError::Empty("host")),
            ("https://example.com:invalid", UrlError::Invalid("port")),
            ("https://example.com:65536", UrlError::Invalid("port")),
            ("http://256.0.0.1", UrlError::Invalid("ipv4 address")),
            ("http://[invalid]", UrlError::Invalid("ipv6 address")),
            ("http://exa mple.com", UrlError::Invalid("domain")),
        ] {
            assert_eq!(
                Url.validate(value),
                Err(ValidationErrors::single(error)),
                "{value:?}"
            );
        }
    }
}
