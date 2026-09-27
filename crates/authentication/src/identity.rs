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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_identifier_whitespace() {
        let issuer = Issuer::try_new(" issuer ").unwrap();
        let subject_id = SubjectId::try_new(" subject ").unwrap();

        let issuer: &str = issuer.as_ref();
        let subject_id: &str = subject_id.as_ref();
        assert_eq!(issuer, " issuer ");
        assert_eq!(subject_id, " subject ");
    }
}
