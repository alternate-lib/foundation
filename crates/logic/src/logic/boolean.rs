use crate::{Conjunction, Disjunction, Logic, Negation};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Boolean;

impl Logic for Boolean {
    type Value = bool;
}

impl Conjunction for Boolean {
    fn and(left: bool, right: bool) -> bool {
        left && right
    }

    fn identity() -> bool {
        true
    }

    fn short_circuit(left: &bool) -> Option<bool> {
        (!*left).then_some(false)
    }
}

impl Disjunction for Boolean {
    fn or(left: bool, right: bool) -> bool {
        left || right
    }

    fn identity() -> bool {
        false
    }

    fn short_circuit(left: &bool) -> Option<bool> {
        (*left).then_some(true)
    }
}

impl Negation for Boolean {
    fn not(value: bool) -> bool {
        !value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_conjunction() {
        assert!(<Boolean as Conjunction>::and(true, true));
        assert!(!<Boolean as Conjunction>::and(true, false));
        assert!(!<Boolean as Conjunction>::and(false, true));
        assert!(!<Boolean as Conjunction>::and(false, false));
    }

    #[test]
    fn short_circuits_conjunction_on_false() {
        assert_eq!(<Boolean as Conjunction>::short_circuit(&true), None);
        assert_eq!(<Boolean as Conjunction>::short_circuit(&false), Some(false));
    }

    #[test]
    fn handles_disjunction() {
        assert!(<Boolean as Disjunction>::or(true, true));
        assert!(<Boolean as Disjunction>::or(true, false));
        assert!(<Boolean as Disjunction>::or(false, true));
        assert!(!<Boolean as Disjunction>::or(false, false));
    }

    #[test]
    fn short_circuits_disjunction_on_true() {
        assert_eq!(<Boolean as Disjunction>::short_circuit(&true), Some(true));
        assert_eq!(<Boolean as Disjunction>::short_circuit(&false), None);
    }

    #[test]
    fn handles_negation() {
        assert!(!<Boolean as Negation>::not(true));
        assert!(<Boolean as Negation>::not(false));
    }
}
