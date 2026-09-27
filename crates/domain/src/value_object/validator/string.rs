pub fn non_empty(value: &str) -> Result<(), NonEmptyStringError> {
    if value.is_empty() {
        return Err(NonEmptyStringError);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("string cannot be empty")]
pub struct NonEmptyStringError;

pub fn min_length<const N: usize>(value: &str) -> Result<(), MinLengthStringError> {
    if value.chars().count() < N {
        return Err(MinLengthStringError(N));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("string cannot be fewer than {0} characters in length")]
pub struct MinLengthStringError(usize);

pub fn max_length<const N: usize>(value: &str) -> Result<(), MaxLengthStringError> {
    if value.chars().count() > N {
        return Err(MaxLengthStringError(N));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("string cannot be greater than {0} characters in length")]
pub struct MaxLengthStringError(usize);

pub fn exact_length<const N: usize>(value: &str) -> Result<(), ExactLengthStringError> {
    if value.chars().count() != N {
        return Err(ExactLengthStringError(N));
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("string must be exactly {0} characters in length")]
pub struct ExactLengthStringError(usize);

pub fn email(value: &str) -> Result<(), EmailError> {
    min_length::<3>(value)?;
    max_length::<255>(value)?;

    if value.chars().any(char::is_whitespace) {
        return Err(EmailError::ContainsWhitespace);
    }

    let parts: Vec<&str> = value.split('@').collect();
    if parts.len() != 2 {
        return Err(EmailError::InvalidFormat);
    }

    let local = parts[0];
    let domain = parts[1];
    if local.is_empty() || domain.is_empty() {
        return Err(EmailError::InvalidFormat);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum EmailError {
    #[error("value contains whitespace")]
    ContainsWhitespace,

    #[error("value has invalid format")]
    InvalidFormat,

    #[error(transparent)]
    TooShort(#[from] MinLengthStringError),

    #[error(transparent)]
    TooLong(#[from] MaxLengthStringError),
}

pub fn url(value: &str) -> Result<(), UrlError> {
    non_empty(value)?;

    if value.chars().any(char::is_whitespace) {
        return Err(UrlError::ContainsWhitespace);
    }

    let Some(scheme_pos) = value.find("://") else {
        return Err(UrlError::MissingScheme);
    };

    let scheme = &value[..scheme_pos];
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return Err(UrlError::InvalidScheme);
    }

    if scheme_pos + 3 >= value.len() {
        return Err(UrlError::MissingAuthority);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UrlError {
    #[error("URL contains whitespace")]
    ContainsWhitespace,

    #[error("URL missing scheme separator")]
    MissingScheme,

    #[error("URL has invalid scheme")]
    InvalidScheme,

    #[error("URL missing authority after scheme")]
    MissingAuthority,

    #[error(transparent)]
    EmptyString(#[from] NonEmptyStringError),
}
