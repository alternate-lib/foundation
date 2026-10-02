use crate::{ValidationErrors, Validator};

pub struct NonEmpty;

impl Validator<str> for NonEmpty {
    type Error = NonEmptyError;

    fn validate(&self, value: &str) -> Result<(), ValidationErrors<Self::Error>> {
        if value.is_empty() {
            return Err(ValidationErrors::single(NonEmptyError));
        }

        Ok(())
    }
}

impl<V> Validator<[V]> for NonEmpty {
    type Error = NonEmptyError;

    fn validate(&self, value: &[V]) -> Result<(), ValidationErrors<Self::Error>> {
        if value.is_empty() {
            return Err(ValidationErrors::single(NonEmptyError));
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("length must be nonzero")]
pub struct NonEmptyError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_empty_strings() {
        assert_eq!(
            NonEmpty.validate(""),
            Err(ValidationErrors::single(NonEmptyError))
        );
        for value in ["a", "🦀", " ", "\n"] {
            assert_eq!(NonEmpty.validate(value), Ok(()));
        }
    }

    #[test]
    fn non_empty_slices() {
        assert_eq!(
            NonEmpty.validate(&[] as &[u8]),
            Err(ValidationErrors::single(NonEmptyError))
        );
        assert_eq!(NonEmpty.validate(&[0][..]), Ok(()));
        assert_eq!(NonEmpty.validate(&[(); 1][..]), Ok(()));
    }
}
