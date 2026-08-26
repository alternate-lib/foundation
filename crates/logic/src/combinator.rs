use crate::{Conjunction, Disjunction, Negation, Predicate};

pub struct And<L, R>(L, R);

impl<L, R> And<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<C, Le, R, Lo> Predicate<C> for And<Le, R>
where
    Le: Predicate<C, Logic = Lo>,
    R: Predicate<C, Logic = Lo>,
    Lo: Conjunction,
{
    type Logic = Lo;

    fn evaluate(&self, context: &C) -> Lo::Value {
        let left = self.0.evaluate(context);

        if let Some(result) = Lo::short_circuit(&left) {
            return result;
        }

        Lo::and(left, self.1.evaluate(context))
    }
}

pub struct Or<L, R>(L, R);

impl<L, R> Or<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<C, Le, R, Lo> Predicate<C> for Or<Le, R>
where
    Le: Predicate<C, Logic = Lo>,
    R: Predicate<C, Logic = Lo>,
    Lo: Disjunction,
{
    type Logic = Lo;

    fn evaluate(&self, context: &C) -> Lo::Value {
        let left = self.0.evaluate(context);

        if let Some(result) = Lo::short_circuit(&left) {
            return result;
        }

        Lo::or(left, self.1.evaluate(context))
    }
}

pub struct Not<P>(P);

impl<P> Not<P> {
    pub const fn new(predicate: P) -> Self {
        Self(predicate)
    }
}

impl<C, P, L> Predicate<C> for Not<P>
where
    P: Predicate<C, Logic = L>,
    L: Negation,
{
    type Logic = L;

    fn evaluate(&self, context: &C) -> L::Value {
        L::not(self.0.evaluate(context))
    }
}

pub struct All<P>(Vec<P>);

impl<P> All<P> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn with_predicate(mut self, predicate: P) -> Self {
        self.0.push(predicate);
        self
    }
}

impl<P> Default for All<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> FromIterator<P> for All<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<C, P, L> Predicate<C> for All<P>
where
    P: Predicate<C, Logic = L>,
    L: Conjunction,
{
    type Logic = L;

    fn evaluate(&self, context: &C) -> L::Value {
        let mut value = L::identity();

        for predicate in &self.0 {
            if let Some(result) = L::short_circuit(&value) {
                return result;
            }

            value = L::and(value, predicate.evaluate(context));
        }

        value
    }
}

pub struct Any<P>(Vec<P>);

impl<P> Any<P> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn with_predicate(mut self, predicate: P) -> Self {
        self.0.push(predicate);
        self
    }
}

impl<P> Default for Any<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> FromIterator<P> for Any<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<C, P, L> Predicate<C> for Any<P>
where
    P: Predicate<C, Logic = L>,
    L: Disjunction,
{
    type Logic = L;

    fn evaluate(&self, context: &C) -> L::Value {
        let mut value = L::identity();

        for predicate in &self.0 {
            if let Some(result) = L::short_circuit(&value) {
                return result;
            }

            value = L::or(value, predicate.evaluate(context));
        }

        value
    }
}
