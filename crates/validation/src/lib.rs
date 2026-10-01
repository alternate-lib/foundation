pub use combinator::{All, And, Any, Or};

pub mod combinator;
mod logic;

pub trait Validator<T: ?Sized> {
    type Error: std::error::Error;

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>>;

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

    fn validate(&self, value: &T) -> Result<(), Vec<Self::Error>> {
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

    fn validate(&self, value: &T) -> Result<(), Vec<E>> {
        (self.0)(value).map_err(|error| vec![error])
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

    fn validate(&self, value: &T) -> Result<(), Vec<E>> {
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

    fn validate(&self, value: &T) -> Result<(), Vec<E>> {
        match self.validator.validate(value) {
            Ok(()) => Err(vec![(self.error)(value)]),
            Err(_) => Ok(()),
        }
    }
}
