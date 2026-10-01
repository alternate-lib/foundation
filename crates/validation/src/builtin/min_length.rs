use crate::Validator;

pub struct MinLength<const L: usize>;

impl<const L: usize> Validator<str> for MinLength<L> {
    type Error = MinLengthError;

    fn validate(&self, value: &str) -> Result<(), Vec<Self::Error>> {
        let actual = value.chars().count();
        if actual < L {
            return Err(vec![MinLengthError { minimum: L, actual }]);
        }

        Ok(())
    }
}

impl<V, const L: usize> Validator<[V]> for MinLength<L> {
    type Error = MinLengthError;

    fn validate(&self, value: &[V]) -> Result<(), Vec<Self::Error>> {
        let actual = value.len();
        if actual < L {
            return Err(vec![MinLengthError { minimum: L, actual }]);
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("length must be at least {minimum}, found {actual}")]
pub struct MinLengthError {
    pub minimum: usize,
    pub actual: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_length_strings() {
        for value in ["abc", "abcd", "é中🦀"] {
            assert_eq!(MinLength::<3>.validate(value), Ok(()));
        }
        for (value, actual) in [("", 0), ("ab", 2), ("é🦀", 2)] {
            assert_eq!(
                MinLength::<3>.validate(value),
                Err(vec![MinLengthError { minimum: 3, actual }]),
                "{value:?}"
            );
        }
    }

    #[test]
    fn min_length_slices() {
        for value in [&[1, 2, 3][..], &[1, 2, 3, 4][..]] {
            assert_eq!(MinLength::<3>.validate(value), Ok(()));
        }
        for (value, actual) in [(&[][..], 0), (&[1, 2][..], 2)] {
            assert_eq!(
                MinLength::<3>.validate(value),
                Err(vec![MinLengthError { minimum: 3, actual }])
            );
        }
        assert_eq!(MinLength::<3>.validate(&[(); 3][..]), Ok(()));
    }

    #[test]
    fn min_length_zero() {
        for value in ["", "🦀"] {
            assert_eq!(MinLength::<0>.validate(value), Ok(()));
        }
        for value in [&[][..], &[1][..]] {
            assert_eq!(MinLength::<0>.validate(value), Ok(()));
        }
    }
}
