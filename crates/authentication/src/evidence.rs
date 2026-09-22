use crate::AuthenticatedIdentity;

pub struct VerifiedCredential<E = ()> {
    identity: AuthenticatedIdentity,
    evidence: E,
}

impl<E> VerifiedCredential<E> {
    pub fn new(identity: AuthenticatedIdentity, evidence: E) -> Self {
        Self { identity, evidence }
    }

    pub fn identity(&self) -> &AuthenticatedIdentity {
        &self.identity
    }

    pub fn evidence(&self) -> &E {
        &self.evidence
    }

    pub fn into_parts(self) -> (AuthenticatedIdentity, E) {
        (self.identity, self.evidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AuthenticatedIdentity, CredentialContext, CredentialKind, CredentialRestrictions, Issuer,
        Subject, SubjectId,
    };

    struct Evidence {
        value: u64,
    }

    fn identity() -> AuthenticatedIdentity {
        AuthenticatedIdentity::new(
            Subject::new(
                Issuer::try_new("https://identity.example").unwrap(),
                SubjectId::try_new("subject-123").unwrap(),
            ),
            CredentialContext::new(
                CredentialKind::Bearer,
                None,
                CredentialRestrictions::unrestricted(),
            ),
        )
    }

    #[test]
    fn exposes_and_round_trips_identity_and_evidence() {
        let credential = VerifiedCredential::new(identity(), Evidence { value: 42 });

        assert_eq!(credential.identity().subject(), identity().subject());
        assert_eq!(credential.evidence().value, 42);

        let (actual_identity, evidence) = credential.into_parts();
        assert_eq!(actual_identity, identity());
        assert_eq!(evidence.value, 42);
    }

    #[test]
    fn supports_unit_evidence() {
        let credential = VerifiedCredential::new(identity(), ());

        let (actual_identity, evidence): (AuthenticatedIdentity, ()) = credential.into_parts();
        assert_eq!(actual_identity, identity());
        assert_eq!(evidence, ());
    }
}
