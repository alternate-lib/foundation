use crate::AuthenticatedIdentity;

pub trait ContextResolver {
    type Scope;
    type Context;

    fn resolve(
        &self,
        identity: AuthenticatedIdentity,
        scope: Self::Scope,
    ) -> impl Future<Output = Result<Self::Context, ResolverError>> + Send;
}

#[derive(Debug, thiserror::Error)]
pub enum ResolverError {
    #[error("application access denied")]
    AccessDenied,
}
