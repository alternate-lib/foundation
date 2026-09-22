use crate::VerifiedCredential;

pub trait ContextResolver {
    type Evidence;
    type Scope;
    type Context;
    type BackendError: std::error::Error;

    fn resolve(
        &self,
        credential: VerifiedCredential<Self::Evidence>,
        scope: Self::Scope,
    ) -> impl Future<Output = Result<Self::Context, ContextResolverError<Self::BackendError>>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum ContextResolverError<E> {
    #[error("application access denied")]
    AccessDenied,

    #[error(transparent)]
    Backend(#[from] E),
}

impl<E> ContextResolverError<E> {
    pub fn map_backend<F>(self, map: impl FnOnce(E) -> F) -> ContextResolverError<F> {
        match self {
            Self::AccessDenied => ContextResolverError::AccessDenied,
            Self::Backend(error) => ContextResolverError::Backend(map(error)),
        }
    }
}
