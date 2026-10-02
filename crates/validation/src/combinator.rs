use alternate_logic::{
    Predicate,
    combinator::{All as LogicAll, And as LogicAnd, Any as LogicAny, Or as LogicOr},
};

use crate::{
    ValidationErrors, Validator,
    logic::{ValidationContext, ValidatorPredicate},
};

pub struct And<L, R>(LogicAnd<ValidatorPredicate<L>, ValidatorPredicate<R>>);

impl<L, R> And<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(LogicAnd::new(
            ValidatorPredicate(left),
            ValidatorPredicate(right),
        ))
    }
}

impl<T: ?Sized, L, R> Validator<T> for And<L, R>
where
    L: Validator<T>,
    R: Validator<T, Error = L::Error>,
{
    type Error = L::Error;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<Self::Error>> {
        self.0.evaluate(&ValidationContext { value })
    }
}

pub struct Or<L, R>(LogicOr<ValidatorPredicate<L>, ValidatorPredicate<R>>);

impl<L, R> Or<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(LogicOr::new(
            ValidatorPredicate(left),
            ValidatorPredicate(right),
        ))
    }
}

impl<T: ?Sized, L, R> Validator<T> for Or<L, R>
where
    L: Validator<T>,
    R: Validator<T, Error = L::Error>,
{
    type Error = L::Error;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<Self::Error>> {
        self.0.evaluate(&ValidationContext { value })
    }
}

pub struct All<V>(LogicAll<ValidatorPredicate<V>>);

impl<V> All<V> {
    pub const fn new() -> Self {
        Self(LogicAll::new())
    }

    #[must_use]
    pub fn with_validator(mut self, validator: V) -> Self {
        self.0 = self.0.with_predicate(ValidatorPredicate(validator));
        self
    }
}

impl<V> Default for All<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> FromIterator<V> for All<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self(iter.into_iter().map(ValidatorPredicate).collect())
    }
}

impl<T: ?Sized, V: Validator<T>> Validator<T> for All<V> {
    type Error = V::Error;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<Self::Error>> {
        self.0.evaluate(&ValidationContext { value })
    }
}

pub struct Any<V>(LogicAny<ValidatorPredicate<V>>);

impl<V> Any<V> {
    pub const fn new() -> Self {
        Self(LogicAny::new())
    }

    #[must_use]
    pub fn with_validator(mut self, validator: V) -> Self {
        self.0 = self.0.with_predicate(ValidatorPredicate(validator));
        self
    }
}

impl<V> Default for Any<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> FromIterator<V> for Any<V> {
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        Self(iter.into_iter().map(ValidatorPredicate).collect())
    }
}

impl<T: ?Sized, V: Validator<T>> Validator<T> for Any<V> {
    type Error = V::Error;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<Self::Error>> {
        self.0.evaluate(&ValidationContext { value })
    }
}

#[macro_export]
macro_rules! all {
    ($validator:expr $(,)?) => {
        $validator
    };
    ($first:expr, $($rest:expr),+ $(,)?) => {
        $crate::And::new($first, $crate::all!($($rest),+))
    };
}

#[macro_export]
macro_rules! any {
    ($validator:expr $(,)?) => {
        $validator
    };
    ($first:expr, $($rest:expr),+ $(,)?) => {
        $crate::Or::new($first, $crate::any!($($rest),+))
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{FnValidator, ValidationErrors, Validator, builtin::MinLength};

    #[derive(Debug, PartialEq, Eq, thiserror::Error)]
    #[error("{0}")]
    struct TestError(&'static str);

    fn tracked<'a>(
        log: &'a RefCell<Vec<&'static str>>,
        name: &'static str,
        valid: bool,
    ) -> impl Validator<(), Error = TestError> + 'a {
        FnValidator::new(move |(): &()| {
            log.borrow_mut().push(name);
            if valid { Ok(()) } else { Err(TestError(name)) }
        })
    }

    #[test]
    fn all_single_validator_is_returned_unchanged() {
        let validator: MinLength<3> = crate::all!(MinLength::<3>);
        assert_eq!(validator.validate("abc"), Ok(()));
        assert!(validator.validate("ab").is_err());

        let validator: MinLength<3> = crate::all!(MinLength::<3>,);
        assert_eq!(validator.validate("abc"), Ok(()));
        assert!(validator.validate("ab").is_err());
    }

    #[test]
    fn any_single_validator_is_returned_unchanged() {
        let validator: MinLength<3> = crate::any!(MinLength::<3>);
        assert_eq!(validator.validate("abc"), Ok(()));
        assert!(validator.validate("ab").is_err());

        let validator: MinLength<3> = crate::any!(MinLength::<3>,);
        assert_eq!(validator.validate("abc"), Ok(()));
        assert!(validator.validate("ab").is_err());
    }

    #[test]
    fn all_requires_every_validator_to_succeed() {
        for first in [false, true] {
            for second in [false, true] {
                for third in [false, true] {
                    let log = RefCell::new(Vec::new());
                    let validator = crate::all!(
                        tracked(&log, "first", first),
                        tracked(&log, "second", second),
                        tracked(&log, "third", third),
                    );
                    let errors: Vec<_> = [("first", first), ("second", second), ("third", third)]
                        .into_iter()
                        .filter_map(|(name, valid)| (!valid).then_some(TestError(name)))
                        .collect();
                    let expected = if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(ValidationErrors::new(errors))
                    };

                    assert_eq!(validator.validate(&()), expected);
                    assert_eq!(*log.borrow(), ["first", "second", "third"]);
                }
            }
        }
    }

    #[test]
    fn any_succeeds_and_short_circuits_at_first_success() {
        for first in [false, true] {
            for second in [false, true] {
                for third in [false, true] {
                    let log = RefCell::new(Vec::new());
                    let validator = crate::any!(
                        tracked(&log, "first", first),
                        tracked(&log, "second", second),
                        tracked(&log, "third", third),
                    );
                    let expected = if first || second || third {
                        Ok(())
                    } else {
                        Err(ValidationErrors::new(vec![
                            TestError("first"),
                            TestError("second"),
                            TestError("third"),
                        ]))
                    };
                    let calls = if first {
                        1
                    } else if second {
                        2
                    } else {
                        3
                    };

                    assert_eq!(validator.validate(&()), expected);
                    assert_eq!(*log.borrow(), ["first", "second", "third"][..calls]);
                }
            }
        }
    }

    #[test]
    fn multiple_validators_accept_no_trailing_comma() {
        let log = RefCell::new(Vec::new());
        let all = crate::all!(tracked(&log, "first", true), tracked(&log, "second", true));
        assert_eq!(all.validate(&()), Ok(()));

        let any = crate::any!(tracked(&log, "first", false), tracked(&log, "second", true));
        assert_eq!(any.validate(&()), Ok(()));
    }
}
