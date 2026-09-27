use std::future::Future;

pub trait Authenticator<I> {
    type Context;
    type Error: std::error::Error;

    fn authenticate(
        &self,
        input: I,
    ) -> impl Future<Output = Result<Self::Context, AuthenticatorError<Self::Error>>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticatorError<E> {
    #[error("credential invalid")]
    InvalidCredential,

    #[error("access denied")]
    AccessDenied,

    #[error(transparent)]
    Backend(E),
}

impl<E> AuthenticatorError<E> {
    pub fn map_backend<F>(self, map: impl FnOnce(E) -> F) -> AuthenticatorError<F> {
        match self {
            Self::InvalidCredential => AuthenticatorError::InvalidCredential,
            Self::AccessDenied => AuthenticatorError::AccessDenied,
            Self::Backend(error) => AuthenticatorError::Backend(map(error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, fmt};

    #[derive(Debug, PartialEq, Eq)]
    struct BackendError(u8);

    impl fmt::Display for BackendError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(formatter, "backend error {}", self.0)
        }
    }

    impl Error for BackendError {}

    #[test]
    fn maps_backend_errors_without_changing_semantic_failures() {
        let backend = AuthenticatorError::Backend(BackendError(1));
        let invalid: AuthenticatorError<BackendError> = AuthenticatorError::InvalidCredential;
        let denied: AuthenticatorError<BackendError> = AuthenticatorError::AccessDenied;

        assert!(matches!(
            backend.map_backend(|error| BackendError(error.0 + 1)),
            AuthenticatorError::Backend(BackendError(2))
        ));
        assert!(matches!(
            invalid.map_backend(|error| error),
            AuthenticatorError::InvalidCredential
        ));
        assert!(matches!(
            denied.map_backend(|error| error),
            AuthenticatorError::AccessDenied
        ));
    }
}
