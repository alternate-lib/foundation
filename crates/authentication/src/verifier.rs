use crate::{AuthenticatedIdentity, Credential};

pub trait CredentialVerifier {
    fn verify(
        &self,
        credential: Credential,
    ) -> impl Future<Output = Result<AuthenticatedIdentity, VerifierError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("credential missing")]
    MissingCredential,

    #[error("credential invalid")]
    InvalidCredential,
}
