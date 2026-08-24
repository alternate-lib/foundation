use crate::{
    AccessRequest, AccessRequestError, Authorized, Policy, PolicyDecision, Subject,
    policy::{All, Any},
};

pub trait HasPermission<A>: Subject {
    type Permission: Eq;

    fn has_permission(&self, permission: &Self::Permission, action: &A) -> bool;
}

pub trait HasPermissionOn<R, A>: Subject {
    type Permission: Eq;

    fn has_permission_on(&self, permission: &Self::Permission, resource: &R, action: &A) -> bool;
}

pub struct RequirePermission<P>(P);

impl<P> RequirePermission<P> {
    pub const fn new(permission: P) -> Self {
        Self(permission)
    }
}

impl<S, R, A, P> Policy<S, R, A> for RequirePermission<P>
where
    S: HasPermission<A, Permission = P>,
{
    fn evaluate(&self, subject: &S, _: &R, action: &A) -> PolicyDecision {
        if subject.has_permission(&self.0, action) {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<S, R, A, P> AccessRequest<S, R, A>
where
    S: HasPermission<A, Permission = P>,
{
    pub fn check_permission(self, permission: P) -> Result<Authorized<R, A>, AccessRequestError> {
        self.authorize(&RequirePermission::new(permission))
    }

    pub fn check_all_permissions(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Authorized<R, A>, AccessRequestError> {
        let policy = All::from_iter(permissions.into_iter().map(RequirePermission::new));

        self.authorize(&policy)
    }

    pub fn check_any_permission(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Authorized<R, A>, AccessRequestError> {
        let policy = Any::from_iter(permissions.into_iter().map(RequirePermission::new));

        self.authorize(&policy)
    }
}

pub struct RequirePermissionOn<P>(P);

impl<P> RequirePermissionOn<P> {
    pub const fn new(permission: P) -> Self {
        Self(permission)
    }
}

impl<S, R, A, P> Policy<S, R, A> for RequirePermissionOn<P>
where
    S: HasPermissionOn<R, A, Permission = P>,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        if subject.has_permission_on(&self.0, resource, action) {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }
}

impl<S, R, A, P> AccessRequest<S, R, A>
where
    S: HasPermissionOn<R, A, Permission = P>,
{
    pub fn check_permission_on(
        self,
        permission: P,
    ) -> Result<Authorized<R, A>, AccessRequestError> {
        self.authorize(&RequirePermissionOn::new(permission))
    }

    pub fn check_all_permissions_on(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Authorized<R, A>, AccessRequestError> {
        let policy = All::from_iter(permissions.into_iter().map(RequirePermissionOn::new));

        self.authorize(&policy)
    }

    pub fn check_any_permission_on(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Authorized<R, A>, AccessRequestError> {
        let policy = Any::from_iter(permissions.into_iter().map(RequirePermissionOn::new));

        self.authorize(&policy)
    }
}
