use crate::{Conjunction, Predicate};

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

        let predicate = And::new(
            tracked(&log, "left", KleeneValue::Unknown),
            tracked(&log, "right", KleeneValue::True),
        );
        predicate.evaluate(&());

        assert_eq!(*log.borrow(), ["left", "right"]);
    }

    #[test]
    fn evaluates_predicate_using_conjunction() {
        for (left, right) in KLEENE_PAIRS {
            let predicate = And::new(
                Constant::<StrongKleene>::new(left),
                Constant::<StrongKleene>::new(right),
            );

            assert_eq!(
                predicate.evaluate(&()),
                <StrongKleene as Conjunction>::and(left, right)
            );
        }
    }

    #[test]
    fn short_circuits_on_dominating_value() {
        let predicate = And::new(
            Constant::<StrongKleene>::new(KleeneValue::False),
            panic_predicate(),
        );

        assert_eq!(predicate.evaluate(&()), KleeneValue::False);
    }

    #[test]
    fn short_circuiting_skips_evaluation() {
        let right_calls = Cell::new(0);
        let right = FnPredicate::<_, StrongKleene>::new(|(): &()| {
            right_calls.set(right_calls.get() + 1);
            KleeneValue::False
        });

        let predicate = And::new(Constant::<StrongKleene>::new(KleeneValue::False), right);
        predicate.evaluate(&());

        assert_eq!(right_calls.get(), 0);
    }
}
