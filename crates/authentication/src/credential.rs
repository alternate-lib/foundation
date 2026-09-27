use std::{collections::BTreeSet, fmt};

#[nutype::nutype(
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref),
    validate(not_empty, len_char_max = SecretCredential::MAX_LENGTH)
)]
pub struct SecretCredential(String);

impl SecretCredential {
    const MAX_LENGTH: usize = 16 * 1024;
}

impl fmt::Debug for SecretCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretCredential([REDACTED])")
    }
}

#[nutype::nutype(
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref),
    validate(not_empty)
)]
pub struct CredentialKind(String);

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
    fn redacts_secrets_from_debug_output() {
        let credential = SecretCredential::try_new("highly-secret-token").unwrap();
        let output = format!("{credential:?}");

        assert!(output.contains("REDACTED"));
        assert!(!output.contains("highly-secret-token"));
    }

    #[test]
    fn preserves_secret_bytes() {
        let credential = SecretCredential::try_new(" secret ").unwrap();

        let value: &str = credential.as_ref();
        assert_eq!(value, " secret ");
    }

    #[test]
    fn preserves_identifier_whitespace() {
        let credential_id = CredentialId::try_new(" credential ").unwrap();
        let scope = CredentialScope::try_new(" scope ").unwrap();

        let credential_id: &str = credential_id.as_ref();
        let scope: &str = scope.as_ref();

        assert_eq!(credential_id, " credential ");
        assert_eq!(scope, " scope ");
    }

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
}
