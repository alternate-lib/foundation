pub trait CredentialVerifier<R> {
    type Verified;
    type Error: std::error::Error;

    fn verify(&self, request: &R) -> Result<Self::Verified, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, fmt};

    struct VerificationRequest<'a> {
        presented: &'a str,
        expected: &'a str,
        now: u64,
        expires_at: u64,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct VerifiedCredential<'a> {
        value: &'a str,
    }

    #[derive(Debug, PartialEq, Eq)]
    enum VerificationError {
        Invalid,
        Expired,
    }

    impl fmt::Display for VerificationError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(match self {
                Self::Invalid => "credential invalid",
                Self::Expired => "credential expired",
            })
        }
    }

    impl Error for VerificationError {}

    struct FakeVerifier;

    impl<'a> CredentialVerifier<VerificationRequest<'a>> for FakeVerifier {
        type Verified = VerifiedCredential<'a>;
        type Error = VerificationError;

        fn verify(&self, request: &VerificationRequest<'a>) -> Result<Self::Verified, Self::Error> {
            if request.presented != request.expected {
                return Err(VerificationError::Invalid);
            }
            if request.now >= request.expires_at {
                return Err(VerificationError::Expired);
            }

            Ok(VerifiedCredential {
                value: request.presented,
            })
        }
    }

    #[test]
    fn verifies_synchronously_against_supplied_facts() {
        let request = VerificationRequest {
            presented: "secret",
            expected: "secret",
            now: 99,
            expires_at: 100,
        };

        assert_eq!(
            FakeVerifier.verify(&request),
            Ok(VerifiedCredential { value: "secret" })
        );
    }

    #[test]
    fn rejects_at_the_expiration_boundary() {
        let request = VerificationRequest {
            presented: "secret",
            expected: "secret",
            now: 100,
            expires_at: 100,
        };

        assert_eq!(
            FakeVerifier.verify(&request),
            Err(VerificationError::Expired)
        );
    }
}
