use std::marker::PhantomData;

use alternate_logic::{Conjunction, Disjunction, Logic, Predicate};

use crate::{ValidationErrors, Validator};

pub(crate) struct ValidationLogic<E>(PhantomData<fn() -> E>);

impl<E> Logic for ValidationLogic<E> {
    type Value = Result<(), ValidationErrors<E>>;
}

impl<E> Conjunction for ValidationLogic<E> {
    fn identity() -> Self::Value {
        Ok(())
    }

    fn and(left: Self::Value, right: Self::Value) -> Self::Value {
        match (left, right) {
            (Ok(()), result) | (result, Ok(())) => result,
            (Err(mut left), Err(right)) => {
                left.extend(right);
                Err(left)
            }
        }
    }
}

impl<E> Disjunction for ValidationLogic<E> {
    fn identity() -> Self::Value {
        Err(ValidationErrors::default())
    }

    fn or(left: Self::Value, right: Self::Value) -> Self::Value {
        match (left, right) {
            (Ok(()), _) | (_, Ok(())) => Ok(()),
            (Err(mut left), Err(right)) => {
                left.extend(right);
                Err(left)
            }
        }
    }

    fn should_short_circuit(left: &Self::Value) -> bool {
        left.is_ok()
    }
}

pub(crate) struct ValidationContext<'a, T: ?Sized> {
    pub(crate) value: &'a T,
}

pub(crate) struct ValidatorPredicate<V>(pub(crate) V);

impl<T: ?Sized, V> Predicate<ValidationContext<'_, T>> for ValidatorPredicate<V>
where
    V: Validator<T>,
{
    type Logic = ValidationLogic<V::Error>;

    fn evaluate(
        &self,
        context: &ValidationContext<'_, T>,
    ) -> Result<(), ValidationErrors<V::Error>> {
        self.0.validate(context.value)
    }
}
