use alternate_logic::{
    Predicate,
    combinator::{All as LogicAll, And as LogicAnd, Any as LogicAny, Or as LogicOr},
};

use crate::{
    Validator,
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

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>> {
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

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>> {
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

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>> {
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

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>> {
        self.0.evaluate(&ValidationContext { value })
    }
}
