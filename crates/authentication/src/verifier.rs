use crate::{AuthenticatedIdentity, Credential};

pub trait CredentialVerifier {
    type BackendError: std::error::Error;

    fn verify(
        &self,
        credential: Credential,
    ) -> impl Future<
        Output = Result<AuthenticatedIdentity, CredentialVerifierError<Self::BackendError>>,
    > + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialVerifierError<E> {
    #[error("credential missing")]
    MissingCredential,

    #[error("credential invalid")]
    InvalidCredential,

    #[error(transparent)]
    Backend(#[from] E),
}

impl<E> CredentialVerifierError<E> {
    pub fn map_backend<F>(self, map: impl FnOnce(E) -> F) -> CredentialVerifierError<F> {
        match self {
            Self::MissingCredential => CredentialVerifierError::MissingCredential,
            Self::InvalidCredential => CredentialVerifierError::InvalidCredential,
            Self::Backend(error) => CredentialVerifierError::Backend(map(error)),
        }
    }
}
