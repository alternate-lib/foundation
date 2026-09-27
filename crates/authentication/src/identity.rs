use std::collections::BTreeSet;

use crate::CredentialKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedIdentity {
    subject: Subject,
    credential: CredentialContext,
}

impl AuthenticatedIdentity {
    pub fn new(subject: Subject, credential: CredentialContext) -> Self {
        Self {
            subject,
            credential,
        }
    }

    pub fn subject(&self) -> &Subject {
        &self.subject
    }

    pub fn credential(&self) -> &CredentialContext {
        &self.credential
    }

    pub fn into_parts(self) -> (Subject, CredentialContext) {
        (self.subject, self.credential)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Subject {
    issuer: Issuer,
    id: SubjectId,
}

impl Subject {
    pub fn new(issuer: Issuer, id: SubjectId) -> Self {
        Self { issuer, id }
    }

    pub fn issuer(&self) -> &Issuer {
        &self.issuer
    }

    pub fn id(&self) -> &SubjectId {
        &self.id
    }

    pub fn into_parts(self) -> (Issuer, SubjectId) {
        (self.issuer, self.id)
    }
}

#[nutype::nutype(
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, Display),
    validate(not_empty, len_char_max = Issuer::MAX_LENGTH)
)]
pub struct Issuer(String);

impl Issuer {
    const MAX_LENGTH: usize = 2048;
}

#[nutype::nutype(
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, Display),
    validate(not_empty, len_char_max = SubjectId::MAX_LENGTH)
)]
pub struct SubjectId(String);

impl SubjectId {
    const MAX_LENGTH: usize = 255;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialContext {
    kind: CredentialKind,
    credential_id: Option<CredentialId>,
    restrictions: CredentialRestrictions,
}

impl CredentialContext {
    pub fn new(
        kind: CredentialKind,
        credential_id: Option<CredentialId>,
        restrictions: CredentialRestrictions,
    ) -> Self {
        Self {
            kind,
            credential_id,
            restrictions,
        }
    }

    pub fn kind(&self) -> &CredentialKind {
        &self.kind
    }

    pub fn credential_id(&self) -> Option<&CredentialId> {
        self.credential_id.as_ref()
    }

    pub fn restrictions(&self) -> &CredentialRestrictions {
        &self.restrictions
    }

    pub fn into_parts(self) -> (CredentialKind, Option<CredentialId>, CredentialRestrictions) {
        (self.kind, self.credential_id, self.restrictions)
    }
}

#[nutype::nutype(
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, Display),
    validate(not_empty, len_char_max = CredentialId::MAX_LENGTH)
)]
pub struct CredentialId(String);

impl CredentialId {
    const MAX_LENGTH: usize = 255;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialRestrictions {
    Unrestricted,
    Restricted(BTreeSet<CredentialScope>),
}

impl CredentialRestrictions {
    pub fn unrestricted() -> Self {
        Self::Unrestricted
    }

    pub fn restricted(scopes: impl IntoIterator<Item = CredentialScope>) -> Self {
        Self::Restricted(scopes.into_iter().collect())
    }

    pub fn scopes(&self) -> Option<&BTreeSet<CredentialScope>> {
        match self {
            Self::Unrestricted => None,
            Self::Restricted(scopes) => Some(scopes),
        }
    }

    pub fn allows(&self, scope: &CredentialScope) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Restricted(scopes) => scopes.contains(scope),
        }
    }
}

impl Default for CredentialRestrictions {
    fn default() -> Self {
        Self::Restricted(BTreeSet::new())
    }
}

#[nutype::nutype(
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, Display),
    validate(not_empty, len_char_max = CredentialScope::MAX_LENGTH)
)]
pub struct CredentialScope(String);

impl CredentialScope {
    const MAX_LENGTH: usize = 255;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_unrestricted_from_empty_restrictions() {
        let scope = CredentialScope::try_new("posts:read").unwrap();
        let unrestricted = CredentialRestrictions::unrestricted();
        let empty = CredentialRestrictions::default();

        assert!(unrestricted.allows(&scope));
        assert!(!empty.allows(&scope));
        assert!(unrestricted.scopes().is_none());
        assert!(empty.scopes().is_some_and(BTreeSet::is_empty));
    }

    #[test]
    fn only_allows_explicit_restricted_scopes() {
        let allowed = CredentialScope::try_new("posts:read").unwrap();
        let denied = CredentialScope::try_new("posts:write").unwrap();
        let restrictions = CredentialRestrictions::restricted([allowed.clone()]);

        assert!(restrictions.allows(&allowed));
        assert!(!restrictions.allows(&denied));
    }

    #[test]
    fn preserves_identifier_whitespace() {
        let issuer = Issuer::try_new(" issuer ").unwrap();
        let subject_id = SubjectId::try_new(" subject ").unwrap();
        let credential_id = CredentialId::try_new(" credential ").unwrap();
        let scope = CredentialScope::try_new(" scope ").unwrap();

        let issuer: &str = issuer.as_ref();
        let subject_id: &str = subject_id.as_ref();
        let credential_id: &str = credential_id.as_ref();
        let scope: &str = scope.as_ref();

        assert_eq!(issuer, " issuer ");
        assert_eq!(subject_id, " subject ");
        assert_eq!(credential_id, " credential ");
        assert_eq!(scope, " scope ");
    }
}
