use crate::{ValidationErrors, Validator};

#[derive(Debug, Clone, Copy, Default)]
pub struct ExactLength<const L: usize>;

impl<const L: usize> Validator<str> for ExactLength<L> {
    type Error = ExactLengthError;

    fn validate(&self, value: &str) -> Result<(), ValidationErrors<Self::Error>> {
        let actual = value.chars().count();
        if actual != L {
            return Err(ValidationErrors::single(ExactLengthError {
                expected: L,
                actual,
            }));
        }

        Ok(())
    }
}

impl<V, const L: usize> Validator<[V]> for ExactLength<L> {
    type Error = ExactLengthError;

    fn validate(&self, value: &[V]) -> Result<(), ValidationErrors<Self::Error>> {
        let actual = value.len();
        if actual != L {
            return Err(ValidationErrors::single(ExactLengthError {
                expected: L,
                actual,
            }));
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("length must equal exactly {expected}, found {actual}")]
pub struct ExactLengthError {
    pub expected: usize,
    pub actual: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_length_strings() {
        for value in ["abc", "é中🦀"] {
            assert_eq!(ExactLength::<3>.validate(value), Ok(()));
        }
        for (value, actual) in [("", 0), ("ab", 2), ("abcd", 4), ("é🦀", 2)] {
            assert_eq!(
                ExactLength::<3>.validate(value),
                Err(ValidationErrors::single(ExactLengthError {
                    expected: 3,
                    actual
                })),
                "{value:?}"
            );
        }
    }

    #[test]
    fn exact_length_slices() {
        assert_eq!(ExactLength::<3>.validate(&[1, 2, 3][..]), Ok(()));
        for (value, actual) in [(&[][..], 0), (&[1, 2][..], 2), (&[1, 2, 3, 4][..], 4)] {
            assert_eq!(
                ExactLength::<3>.validate(value),
                Err(ValidationErrors::single(ExactLengthError {
                    expected: 3,
                    actual
                }))
            );
        }

        assert_eq!(ExactLength::<3>.validate(&[(); 3][..]), Ok(()));
    }

    #[test]
    fn exact_length_zero() {
        assert_eq!(ExactLength::<0>.validate(""), Ok(()));
        assert_eq!(ExactLength::<0>.validate(&[] as &[u8]), Ok(()));

        let error = Err(ValidationErrors::single(ExactLengthError {
            expected: 0,
            actual: 1,
        }));

        assert_eq!(ExactLength::<0>.validate("🦀"), error);
        assert_eq!(ExactLength::<0>.validate(&[1][..]), error);
    }
}
