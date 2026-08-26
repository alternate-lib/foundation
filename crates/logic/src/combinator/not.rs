use crate::{Negation, Predicate};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        logic::{KleeneValue, StrongKleene},
        predicate::Constant,
    };

    #[test]
    fn evaluates_predicate_using_negation() {
        for value in [KleeneValue::True, KleeneValue::False, KleeneValue::Unknown] {
            let predicate = Not::new(Constant::<StrongKleene>::new(value));

            assert_eq!(
                predicate.evaluate(&()),
                <StrongKleene as Negation>::not(value)
            );
        }
    }
}
