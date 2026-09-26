use crate::VerifiedCredential;

pub trait CredentialVerifier {
    type Credential;
    type Evidence;
    type BackendError: std::error::Error;

    fn verify(
        &self,
        credential: Self::Credential,
    ) -> impl Future<
        Output = Result<
            VerifiedCredential<Self::Evidence>,
            CredentialVerifierError<Self::BackendError>,
        >,
    > + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum CredentialVerifierError<E> {
    #[error("credential invalid")]
    InvalidCredential,

    #[error(transparent)]
    Backend(#[from] E),
}

impl<E> CredentialVerifierError<E> {
    pub fn map_backend<F>(self, map: impl FnOnce(E) -> F) -> CredentialVerifierError<F> {
        match self {
            Self::InvalidCredential => CredentialVerifierError::InvalidCredential,
            Self::Backend(error) => CredentialVerifierError::Backend(map(error)),
        }
    }
}
