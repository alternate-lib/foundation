use std::fmt;

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
}
