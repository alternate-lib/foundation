use std::{fmt, ops::Deref, vec::IntoIter};

pub use combinator::{All, And, Any, Or};

pub mod builtin;
pub mod combinator;
mod logic;

pub trait Validator<T: ?Sized> {
    type Error: std::error::Error;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<Self::Error>>;

    fn map_err<F, E>(self, func: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: Fn(Self::Error) -> E,
    {
        MapErr {
            validator: self,
            func,
        }
    }
}

impl<T: ?Sized, V: Validator<T> + ?Sized> Validator<T> for Box<V> {
    type Error = V::Error;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<Self::Error>> {
        (**self).validate(value)
    }
}

pub struct FnValidator<F>(F);

impl<F> FnValidator<F> {
    pub const fn new(func: F) -> Self {
        Self(func)
    }
}

impl<T: ?Sized, F, E> Validator<T> for FnValidator<F>
where
    F: Fn(&T) -> Result<(), E>,
    E: std::error::Error,
{
    type Error = E;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<E>> {
        (self.0)(value).map_err(ValidationErrors::single)
    }
}

pub struct MapErr<V, F> {
    validator: V,
    func: F,
}

impl<V, F> MapErr<V, F> {
    pub const fn new(validator: V, func: F) -> Self {
        Self { validator, func }
    }
}

impl<T: ?Sized, V, F, E> Validator<T> for MapErr<V, F>
where
    V: Validator<T>,
    F: Fn(V::Error) -> E,
    E: std::error::Error,
{
    type Error = E;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<E>> {
        self.validator
            .validate(value)
            .map_err(|errors| errors.into_iter().map(&self.func).collect())
    }
}

pub struct Not<V, F> {
    validator: V,
    error: F,
}

impl<V, F> Not<V, F> {
    pub const fn new(validator: V, error: F) -> Self {
        Self { validator, error }
    }
}

impl<T: ?Sized, V, F, E> Validator<T> for Not<V, F>
where
    V: Validator<T>,
    F: Fn(&T) -> E,
    E: std::error::Error,
{
    type Error = E;

    fn validate(&self, value: &T) -> Result<(), ValidationErrors<E>> {
        match self.validator.validate(value) {
            Ok(()) => Err(ValidationErrors::single((self.error)(value))),
            Err(_) => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationErrors<E>(Vec<E>);

impl<E> ValidationErrors<E> {
    pub fn new(errors: Vec<E>) -> Self {
        Self(errors)
    }

    pub fn single(error: E) -> Self {
        Self::new(vec![error])
    }

    pub fn into_inner(self) -> Vec<E> {
        self.0
    }
}

impl<E> Default for ValidationErrors<E> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<E> Deref for ValidationErrors<E> {
    type Target = [E];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> From<Vec<E>> for ValidationErrors<E> {
    fn from(value: Vec<E>) -> Self {
        Self(value)
    }
}

impl<E> FromIterator<E> for ValidationErrors<E> {
    fn from_iter<I: IntoIterator<Item = E>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<E> IntoIterator for ValidationErrors<E> {
    type Item = E;
    type IntoIter = IntoIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<E> Extend<E> for ValidationErrors<E> {
    fn extend<I: IntoIterator<Item = E>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<E: fmt::Display> fmt::Display for ValidationErrors<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("validation failed")?;
        for (index, error) in self.iter().enumerate() {
            f.write_str(if index == 0 { ": " } else { "; " })?;
            error.fmt(f)?;
        }

        Ok(())
    }
}

impl<E: std::error::Error> std::error::Error for ValidationErrors<E> {}
