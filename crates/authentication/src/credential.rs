use std::fmt;

#[derive(Clone)]
pub enum Credential {
    Bearer(SecretCredential),
    ApiKey(SecretCredential),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CredentialKind {
    Bearer,
    ApiKey,
}

impl Credential {
    pub fn bearer(secret: impl Into<String>) -> Result<Self, SecretCredentialError> {
        SecretCredential::try_new(secret).map(Self::Bearer)
    }

    pub fn api_key(secret: impl Into<String>) -> Result<Self, SecretCredentialError> {
        SecretCredential::try_new(secret).map(Self::ApiKey)
    }

    pub fn kind(&self) -> CredentialKind {
        match self {
            Self::Bearer(_) => CredentialKind::Bearer,
            Self::ApiKey(_) => CredentialKind::ApiKey,
        }
    }

    pub fn expose_secret(&self) -> &str {
        match self {
            Self::Bearer(secret) | Self::ApiKey(secret) => secret.as_ref(),
        }
    }

    pub fn into_secret(self) -> SecretCredential {
        match self {
            Self::Bearer(secret) | Self::ApiKey(secret) => secret,
        }
    }
}

impl fmt::Debug for Credential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Credential")
            .field("kind", &self.kind())
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

#[nutype::nutype(
    derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref),
    sanitize(trim),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_secrets_from_debug_output() {
        let credential = Credential::bearer("highly-secret-token").unwrap();
        let output = format!("{credential:?}");

        assert!(output.contains("REDACTED"));
        assert!(!output.contains("highly-secret-token"));
        assert!(!format!("{:?}", credential.into_secret()).contains("highly-secret-token"));
    }
}
