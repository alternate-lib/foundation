use crate::{ValidationErrors, Validator};

#[derive(Debug, Clone, Copy, Default)]
pub struct MaxLength<const L: usize>;

impl<const L: usize> Validator<str> for MaxLength<L> {
    type Error = MaxLengthError;

    fn validate(&self, value: &str) -> Result<(), ValidationErrors<Self::Error>> {
        let actual = value.chars().count();
        if actual > L {
            return Err(ValidationErrors::single(MaxLengthError {
                maximum: L,
                actual,
            }));
        }

        Ok(())
    }
}

impl<V, const L: usize> Validator<[V]> for MaxLength<L> {
    type Error = MaxLengthError;

    fn validate(&self, value: &[V]) -> Result<(), ValidationErrors<Self::Error>> {
        let actual = value.len();
        if actual > L {
            return Err(ValidationErrors::single(MaxLengthError {
                maximum: L,
                actual,
            }));
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("length must be at most {maximum}, found {actual}")]
pub struct MaxLengthError {
    pub maximum: usize,
    pub actual: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_length_strings() {
        for value in ["", "ab", "abc", "é中🦀"] {
            assert_eq!(MaxLength::<3>.validate(value), Ok(()));
        }
        for value in ["abcd", "é中🦀ß"] {
            assert_eq!(
                MaxLength::<3>.validate(value),
                Err(ValidationErrors::single(MaxLengthError {
                    maximum: 3,
                    actual: 4
                })),
                "{value:?}"
            );
        }
    }

    #[test]
    fn max_length_slices() {
        for value in [&[][..], &[1, 2][..], &[1, 2, 3][..]] {
            assert_eq!(MaxLength::<3>.validate(value), Ok(()));
        }
        assert_eq!(
            MaxLength::<3>.validate(&[1, 2, 3, 4][..]),
            Err(ValidationErrors::single(MaxLengthError {
                maximum: 3,
                actual: 4
            }))
        );
        assert_eq!(MaxLength::<3>.validate(&[(); 3][..]), Ok(()));
    }

    #[test]
    fn max_length_zero() {
        assert_eq!(MaxLength::<0>.validate(""), Ok(()));
        assert_eq!(MaxLength::<0>.validate(&[] as &[u8]), Ok(()));
        let error = Err(ValidationErrors::single(MaxLengthError {
            maximum: 0,
            actual: 1,
        }));
        assert_eq!(MaxLength::<0>.validate("🦀"), error);
        assert_eq!(MaxLength::<0>.validate(&[1][..]), error);
    }
}
