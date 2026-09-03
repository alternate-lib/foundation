use std::str::FromStr;

#[nutype::nutype(
    derive(Debug, Clone, PartialEq, Eq, Hash, Deref, Display),
    sanitize(trim),
    validate(with = Url::validate_url, error = UrlError),
)]
pub struct Url(String);

impl Url {
    fn validate_url(v: &str) -> Result<(), UrlError> {
        if v.is_empty() {
            return Err(UrlError::EmptyUrl);
        }

        if v.chars().any(char::is_whitespace) {
            return Err(UrlError::ContainsWhitespace);
        }

        let Some(scheme_pos) = v.find("://") else {
            return Err(UrlError::MissingScheme);
        };

        let scheme = &v[..scheme_pos];
        if scheme.is_empty()
            || !scheme
                .chars()
                .all(|c| c.is_alphanumeric() || c == '+' || c == '-' || c == '.')
        {
            return Err(UrlError::InvalidScheme);
        }

        if scheme_pos + 3 >= v.len() {
            return Err(UrlError::MissingAuthority);
        }

        Ok(())
    }
}

impl FromStr for Url {
    type Err = UrlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum UrlError {
    #[error("URL cannot be empty")]
    EmptyUrl,

    #[error("URL contains whitespace")]
    ContainsWhitespace,

    #[error("URL missing scheme separator")]
    MissingScheme,

    #[error("URL has invalid scheme")]
    InvalidScheme,

    #[error("URL missing authority after scheme")]
    MissingAuthority,
}
