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

    fn should_short_circuit(left: &bool) -> bool {
        !*left
    }
}

impl Disjunction for Boolean {
    fn or(left: bool, right: bool) -> bool {
        left || right
    }

    fn identity() -> bool {
        false
    }

    fn should_short_circuit(left: &bool) -> bool {
        *left
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
        assert!(!<Boolean as Conjunction>::should_short_circuit(&true));
        assert!(<Boolean as Conjunction>::should_short_circuit(&false));
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
        assert!(<Boolean as Disjunction>::should_short_circuit(&true));
        assert!(!<Boolean as Disjunction>::should_short_circuit(&false));
    }

    #[test]
    fn handles_negation() {
        assert!(!<Boolean as Negation>::not(true));
        assert!(<Boolean as Negation>::not(false));
    }
}
