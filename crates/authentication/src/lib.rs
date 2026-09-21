pub use credential::{Credential, CredentialKind, SecretCredential};
pub use identity::{
    AuthenticatedIdentity, CredentialContext, CredentialId, CredentialRestrictions,
    CredentialScope, Issuer, Subject, SubjectId,
};
pub use resolver::{ContextResolver, ContextResolverError};
pub use verifier::{CredentialVerifier, CredentialVerifierError};

pub mod credential;
pub mod identity;
pub mod resolver;
pub mod verifier;
