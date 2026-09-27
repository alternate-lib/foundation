pub fn non_empty<T>(value: &[T]) -> Result<(), NonEmptyVecError> {
    if value.is_empty() {
        return Err(NonEmptyVecError);
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("vector cannot be empty")]
pub struct NonEmptyVecError;
