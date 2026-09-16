pub use credential::{Credential, CredentialKind, SecretCredential};
pub use identity::{
    AuthenticatedIdentity, CredentialContext, CredentialId, CredentialRestrictions,
    CredentialScope, Issuer, Subject, SubjectId,
};
pub use resolver::{ContextResolver, ResolverError};
pub use verifier::{CredentialVerifier, VerifierError};

pub mod credential;
pub mod identity;
pub mod resolver;
pub mod verifier;
