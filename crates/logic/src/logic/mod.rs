pub use boolean::Boolean;
pub use kleene::{KleeneMapping, KleeneValue, StrongKleene};

mod boolean;
mod kleene;

/// Identifies a domain's truth-value type.
pub trait Logic {
    /// The truth value evaluated by predicates.
    type Value;
}

pub trait Conjunction: Logic {
    fn and(left: Self::Value, right: Self::Value) -> Self::Value;

    fn identity() -> Self::Value;

    fn should_short_circuit(_left: &Self::Value) -> bool {
        false
    }
}

pub trait Disjunction: Logic {
    fn or(left: Self::Value, right: Self::Value) -> Self::Value;

    fn identity() -> Self::Value;

    fn should_short_circuit(_left: &Self::Value) -> bool {
        false
    }
}

pub trait Negation: Logic {
    fn not(value: Self::Value) -> Self::Value;
}
