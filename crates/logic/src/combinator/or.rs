use crate::{Disjunction, Predicate};

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

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use super::*;
    use crate::{
        combinator::tests::*,
        logic::{KleeneValue, StrongKleene},
        predicate::{Constant, FnPredicate},
    };

    #[test]
    fn evaluates_left_to_right() {
        let log = RefCell::new(Vec::new());

        let predicate = Or::new(
            tracked(&log, "left", KleeneValue::Unknown),
            tracked(&log, "right", KleeneValue::False),
        );
        predicate.evaluate(&());

        assert_eq!(*log.borrow(), ["left", "right"]);
    }

    #[test]
    fn evaluates_predicate_using_disjunction() {
        for (left, right) in KLEENE_PAIRS {
            let predicate = Or::new(
                Constant::<StrongKleene>::new(left),
                Constant::<StrongKleene>::new(right),
            );

            assert_eq!(
                predicate.evaluate(&()),
                <StrongKleene as Disjunction>::or(left, right)
            );
        }
    }

    #[test]
    fn short_circuits_on_dominating_value() {
        let predicate = Or::new(
            Constant::<StrongKleene>::new(KleeneValue::True),
            panic_predicate(),
        );

        assert_eq!(predicate.evaluate(&()), KleeneValue::True);
    }

    #[test]
    fn short_circuiting_skips_evaluation() {
        let right_calls = Cell::new(0);
        let right = FnPredicate::<_, StrongKleene>::new(|(): &()| {
            right_calls.set(right_calls.get() + 1);
            KleeneValue::True
        });

        let predicate = Or::new(Constant::<StrongKleene>::new(KleeneValue::True), right);
        predicate.evaluate(&());

        assert_eq!(right_calls.get(), 0);
    }
}
