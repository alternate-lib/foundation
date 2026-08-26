pub use all::All;
pub use and::And;
pub use any::Any;
pub use not::Not;
pub use or::Or;

mod all;
mod and;
mod any;
mod not;
mod or;

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use crate::{
        logic::{KleeneValue, StrongKleene},
        predicate::FnPredicate,
    };

    pub const KLEENE_PAIRS: [(KleeneValue, KleeneValue); 9] = [
        (KleeneValue::True, KleeneValue::True),
        (KleeneValue::True, KleeneValue::False),
        (KleeneValue::True, KleeneValue::Unknown),
        (KleeneValue::False, KleeneValue::True),
        (KleeneValue::False, KleeneValue::False),
        (KleeneValue::False, KleeneValue::Unknown),
        (KleeneValue::Unknown, KleeneValue::True),
        (KleeneValue::Unknown, KleeneValue::False),
        (KleeneValue::Unknown, KleeneValue::Unknown),
    ];

    pub fn panic_predicate() -> FnPredicate<fn(&()) -> KleeneValue, StrongKleene> {
        FnPredicate::<_, StrongKleene>::new(|(): &()| -> KleeneValue {
            panic!("unexpected evaluation")
        })
    }

    pub fn tracked<'a>(
        log: &'a RefCell<Vec<&'static str>>,
        name: &'static str,
        value: KleeneValue,
    ) -> FnPredicate<impl Fn(&()) -> KleeneValue + 'a, StrongKleene> {
        FnPredicate::<_, StrongKleene>::new(move |(): &()| {
            log.borrow_mut().push(name);
            value
        })
    }
}
