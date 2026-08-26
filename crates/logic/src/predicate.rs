use std::marker::PhantomData;

use crate::Logic;

pub trait Predicate<C> {
    /// The logic whose truth values this predicate produces.
    type Logic: Logic;

    fn evaluate(&self, context: &C) -> <Self::Logic as Logic>::Value;
}

/// A predicate that always yields a fixed truth value.
pub struct Constant<G: Logic> {
    value: G::Value,
}

impl<G: Logic> Constant<G> {
    pub const fn new(value: G::Value) -> Self {
        Self { value }
    }
}

impl<C, G> Predicate<C> for Constant<G>
where
    G: Logic,
    G::Value: Clone,
{
    type Logic = G;

    fn evaluate(&self, _: &C) -> G::Value {
        self.value.clone()
    }
}

/// A predicate backed by a closure. Useful for leaf predicates defined inline.
pub struct FnPredicate<F, G> {
    f: F,
    marker: PhantomData<fn() -> G>,
}

impl<F, G> FnPredicate<F, G> {
    pub const fn new(f: F) -> Self {
        Self {
            f,
            marker: PhantomData,
        }
    }
}

impl<C, F, G> Predicate<C> for FnPredicate<F, G>
where
    F: Fn(&C) -> G::Value,
    G: Logic,
{
    type Logic = G;

    fn evaluate(&self, context: &C) -> G::Value {
        (self.f)(context)
    }
}
