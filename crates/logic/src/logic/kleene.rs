use std::marker::PhantomData;

use crate::{Conjunction, Disjunction, Logic, Negation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KleeneValue {
    True,
    False,
    Unknown,
}

pub trait KleeneMapping {
    fn to_kleene(&self) -> KleeneValue;

    fn from_kleene(value: KleeneValue) -> Self;
}

impl KleeneMapping for KleeneValue {
    fn to_kleene(&self) -> KleeneValue {
        *self
    }

    fn from_kleene(value: KleeneValue) -> Self {
        value
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StrongKleene<V = KleeneValue>(PhantomData<fn() -> V>);

impl<V: KleeneMapping> Logic for StrongKleene<V> {
    type Value = V;
}

impl<V: KleeneMapping> Conjunction for StrongKleene<V> {
    fn and(left: V, right: V) -> V {
        let value = match (left.to_kleene(), right.to_kleene()) {
            (KleeneValue::False, _) | (_, KleeneValue::False) => KleeneValue::False,
            (KleeneValue::True, KleeneValue::True) => KleeneValue::True,
            _ => KleeneValue::Unknown,
        };

        V::from_kleene(value)
    }

    fn identity() -> V {
        V::from_kleene(KleeneValue::True)
    }

    fn short_circuit(left: &V) -> Option<V> {
        (left.to_kleene() == KleeneValue::False).then(|| V::from_kleene(KleeneValue::False))
    }
}

impl<V: KleeneMapping> Disjunction for StrongKleene<V> {
    fn or(left: V, right: V) -> V {
        let value = match (left.to_kleene(), right.to_kleene()) {
            (KleeneValue::True, _) | (_, KleeneValue::True) => KleeneValue::True,
            (KleeneValue::False, KleeneValue::False) => KleeneValue::False,
            _ => KleeneValue::Unknown,
        };

        V::from_kleene(value)
    }

    fn identity() -> V {
        V::from_kleene(KleeneValue::False)
    }

    fn short_circuit(left: &V) -> Option<V> {
        (left.to_kleene() == KleeneValue::True).then(|| V::from_kleene(KleeneValue::True))
    }
}

impl<V: KleeneMapping> Negation for StrongKleene<V> {
    fn not(value: V) -> V {
        V::from_kleene(match value.to_kleene() {
            KleeneValue::False => KleeneValue::True,
            KleeneValue::Unknown => KleeneValue::Unknown,
            KleeneValue::True => KleeneValue::False,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_conjunction() {
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::True, KleeneValue::True),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::True, KleeneValue::False),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::True, KleeneValue::Unknown),
            KleeneValue::Unknown
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::False, KleeneValue::True),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::False, KleeneValue::False),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::False, KleeneValue::Unknown),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::Unknown, KleeneValue::True),
            KleeneValue::Unknown
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::Unknown, KleeneValue::False),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Conjunction>::and(KleeneValue::Unknown, KleeneValue::Unknown),
            KleeneValue::Unknown
        );
    }

    #[test]
    fn short_circuits_conjunction_on_false() {
        assert_eq!(
            <StrongKleene as Conjunction>::short_circuit(&KleeneValue::True),
            None
        );
        assert_eq!(
            <StrongKleene as Conjunction>::short_circuit(&KleeneValue::False),
            Some(KleeneValue::False)
        );
        assert_eq!(
            <StrongKleene as Conjunction>::short_circuit(&KleeneValue::Unknown),
            None
        );
    }

    #[test]
    fn handles_disjunction() {
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::True, KleeneValue::True),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::True, KleeneValue::False),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::True, KleeneValue::Unknown),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::False, KleeneValue::True),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::False, KleeneValue::False),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::False, KleeneValue::Unknown),
            KleeneValue::Unknown
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::Unknown, KleeneValue::True),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::Unknown, KleeneValue::False),
            KleeneValue::Unknown
        );
        assert_eq!(
            <StrongKleene as Disjunction>::or(KleeneValue::Unknown, KleeneValue::Unknown),
            KleeneValue::Unknown
        );
    }

    #[test]
    fn short_circuits_disjunction_on_true() {
        assert_eq!(
            <StrongKleene as Disjunction>::short_circuit(&KleeneValue::True),
            Some(KleeneValue::True)
        );
        assert_eq!(
            <StrongKleene as Disjunction>::short_circuit(&KleeneValue::False),
            None
        );
        assert_eq!(
            <StrongKleene as Disjunction>::short_circuit(&KleeneValue::Unknown),
            None
        );
    }

    #[test]
    fn handles_negation() {
        assert_eq!(
            <StrongKleene as Negation>::not(KleeneValue::True),
            KleeneValue::False
        );
        assert_eq!(
            <StrongKleene as Negation>::not(KleeneValue::False),
            KleeneValue::True
        );
        assert_eq!(
            <StrongKleene as Negation>::not(KleeneValue::Unknown),
            KleeneValue::Unknown
        );
    }
}
