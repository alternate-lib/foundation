use crate::{Conjunction, Predicate};

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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::{
        combinator::tests::*,
        logic::{KleeneValue, StrongKleene},
        predicate::{Constant, FnPredicate},
    };

    #[test]
    fn evaluates_left_to_right() {
        let log = RefCell::new(Vec::new());

        let predicate = All::from_iter([
            tracked(&log, "first", KleeneValue::Unknown),
            tracked(&log, "second", KleeneValue::True),
            tracked(&log, "third", KleeneValue::True),
        ]);
        predicate.evaluate(&());

        assert_eq!(*log.borrow(), ["first", "second", "third"]);
    }

    #[test]
    fn evaluates_predicate_using_conjunction() {
        for (left, right) in KLEENE_PAIRS {
            let predicate = All::from_iter([
                Constant::<StrongKleene>::new(left),
                Constant::<StrongKleene>::new(right),
            ]);

            assert_eq!(
                predicate.evaluate(&()),
                <StrongKleene as Conjunction>::and(left, right)
            );
        }
    }

    #[test]
    fn evaluates_to_conjunction_identity_when_empty() {
        let predicate = All::<Constant<StrongKleene>>::new();

        assert_eq!(
            predicate.evaluate(&()),
            <StrongKleene as Conjunction>::identity()
        );
    }

    #[test]
    fn short_circuits_skipping_later_predicates() {
        let predicate = All::from_iter([
            FnPredicate::<fn(&()) -> KleeneValue, StrongKleene>::new(|(): &()| KleeneValue::False),
            panic_predicate(),
        ]);

        assert_eq!(predicate.evaluate(&()), KleeneValue::False);
    }
}
