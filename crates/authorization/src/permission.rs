use crate::{
    ActionRequest, Grant, Policy, PolicyDecision, RequestError, ResourceRequest, Subject,
    policy::{All, Any},
};

pub trait HasPermission<A>: Subject {
    type Permission: Eq;

    fn has_permission(&self, permission: &Self::Permission, action: &A) -> bool;
}

pub trait HasPermissionOn<A, R>: Subject {
    type Permission: Eq;

    fn has_permission_on(&self, permission: &Self::Permission, action: &A, resource: &R) -> bool;
}

pub struct RequirePermission<P>(P);

impl<P> RequirePermission<P> {
    pub const fn new(permission: P) -> Self {
        Self(permission)
    }
}

impl<S, A, R, P> Policy<S, A, R> for RequirePermission<P>
where
    S: HasPermission<A, Permission = P>,
{
    fn evaluate(&self, subject: &S, action: &A, _: &R) -> PolicyDecision {
        if subject.has_permission(&self.0, action) {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<S, A, P> ActionRequest<S, A>
where
    S: HasPermission<A, Permission = P>,
{
    pub fn check_permission(self, permission: P) -> Result<Grant<S, A>, RequestError> {
        self.authorize(&RequirePermission::new(permission))
    }

    pub fn check_all_permissions(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Grant<S, A>, RequestError> {
        let policy = All::from_iter(permissions.into_iter().map(RequirePermission::new));

        self.authorize(&policy)
    }

    pub fn check_any_permission(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Grant<S, A>, RequestError> {
        let policy = Any::from_iter(permissions.into_iter().map(RequirePermission::new));

        self.authorize(&policy)
    }
}

impl<S, A, R, P> ResourceRequest<S, A, R>
where
    S: HasPermission<A, Permission = P>,
{
    pub fn check_permission(self, permission: P) -> Result<Grant<S, A, R>, RequestError> {
        self.authorize(&RequirePermission::new(permission))
    }

    pub fn check_all_permissions(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Grant<S, A, R>, RequestError> {
        let policy = All::from_iter(permissions.into_iter().map(RequirePermission::new));

        self.authorize(&policy)
    }

    pub fn check_any_permission(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Grant<S, A, R>, RequestError> {
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

impl<S, A, R, P> Policy<S, A, R> for RequirePermissionOn<P>
where
    S: HasPermissionOn<A, R, Permission = P>,
{
    fn evaluate(&self, subject: &S, action: &A, resource: &R) -> PolicyDecision {
        if subject.has_permission_on(&self.0, action, resource) {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }
}

impl<S, A, R, P> ResourceRequest<S, A, R>
where
    S: HasPermissionOn<A, R, Permission = P>,
{
    pub fn check_permission_on(self, permission: P) -> Result<Grant<S, A, R>, RequestError> {
        self.authorize(&RequirePermissionOn::new(permission))
    }

    pub fn check_all_permissions_on(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Grant<S, A, R>, RequestError> {
        let policy = All::from_iter(permissions.into_iter().map(RequirePermissionOn::new));

        self.authorize(&policy)
    }

    pub fn check_any_permission_on(
        self,
        permissions: impl IntoIterator<Item = P>,
    ) -> Result<Grant<S, A, R>, RequestError> {
        let policy = Any::from_iter(permissions.into_iter().map(RequirePermissionOn::new));

        self.authorize(&policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessRequest, test_utils::*};

    #[test]
    fn grants_access_on_possessing_action_permission() {
        let result = AccessRequest::for_subject(User::new(1).with_permission(Permission::Write))
            .performing_action("write")
            .check_permission(Permission::Write);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_action_permission() {
        let err = AccessRequest::for_subject(User::new(2).with_permission(Permission::Read))
            .performing_action("write")
            .check_permission(Permission::Write)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_all_action_permissions() {
        let result = AccessRequest::for_subject(
            User::new(1)
                .with_permission(Permission::Read)
                .with_permission(Permission::Write),
        )
        .performing_action("edit")
        .check_all_permissions([Permission::Read, Permission::Write]);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_some_action_permissions() {
        let err = AccessRequest::for_subject(User::new(1).with_permission(Permission::Read))
            .performing_action("edit")
            .check_all_permissions([Permission::Read, Permission::Write])
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_any_action_permission() {
        let result = AccessRequest::for_subject(User::new(1).with_permission(Permission::Read))
            .performing_action("edit")
            .check_any_permission([Permission::Read, Permission::Write]);

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_on_possessing_resource_permission() {
        let user_id = 1;

        let result = AccessRequest::for_subject(User::new(1).with_permission(Permission::Write))
            .performing_action("write")
            .on_resource(Post::with_owner(user_id))
            .check_permission_on(Permission::Write);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_resource_permission() {
        let user_id = 1;

        let err = AccessRequest::for_subject(User::new(user_id).with_permission(Permission::Read))
            .performing_action("write")
            .on_resource(Post::with_owner(user_id))
            .check_permission(Permission::Write)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_all_resource_permissions() {
        let user_id = 1;

        let result = AccessRequest::for_subject(
            User::new(user_id)
                .with_permission(Permission::Read)
                .with_permission(Permission::Write),
        )
        .performing_action("edit")
        .on_resource(Post::with_owner(user_id))
        .check_all_permissions_on([Permission::Read, Permission::Write]);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_some_resource_permissions() {
        let err = AccessRequest::for_subject(User::new(1).with_permission(Permission::Write))
            .performing_action("edit")
            .on_resource(Post::with_owner(2))
            .check_all_permissions_on([Permission::Read, Permission::Write])
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_any_resource_permission() {
        let result = AccessRequest::for_subject(User::new(1).with_permission(Permission::Read))
            .performing_action("edit")
            .on_resource(Post::with_owner(2))
            .check_any_permission_on([Permission::Read, Permission::Write]);

        assert!(result.is_ok());
    }
}
