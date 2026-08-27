use crate::{ActionRequest, Grant, Policy, PolicyDecision, RequestError, ResourceRequest};

pub trait Action {
    type Permission: Eq;

    fn required_permission(&self) -> Self::Permission;
}

pub trait HasPermission {
    type Permission: Eq;

    fn has_permission(&self, permission: &Self::Permission) -> bool;
}

pub trait HasPermissionOn<R> {
    type Permission: Eq;

    fn has_permission_on(&self, permission: &Self::Permission, resource: &R) -> bool;
}

pub struct RequirePermission;

impl<P, A> Policy<ActionRequest<P, A>> for RequirePermission
where
    P: HasPermission<Permission = A::Permission>,
    A: Action,
{
    fn evaluate(&self, request: &ActionRequest<P, A>) -> PolicyDecision {
        if request
            .principal
            .has_permission(&request.action.required_permission())
        {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<P, A> ActionRequest<P, A>
where
    P: HasPermission<Permission = A::Permission>,
    A: Action,
{
    pub fn check_action_permission(self) -> Result<Grant<P, A>, RequestError> {
        self.authorize(&RequirePermission)
    }
}

pub struct RequirePermissionOn;

impl<P, A, R> Policy<ResourceRequest<P, A, R>> for RequirePermissionOn
where
    P: HasPermissionOn<R, Permission = A::Permission>,
    A: Action,
{
    fn evaluate(&self, request: &ResourceRequest<P, A, R>) -> PolicyDecision {
        if request
            .principal
            .has_permission_on(&request.action.required_permission(), &request.resource)
        {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }
}

impl<P, A, R> ResourceRequest<P, A, R>
where
    P: HasPermissionOn<R, Permission = A::Permission>,
    A: Action,
{
    pub fn check_resource_permission(self) -> Result<Grant<P, A, R>, RequestError> {
        self.authorize(&RequirePermissionOn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessRequest, test_utils::*};

    #[test]
    fn grants_access_on_possessing_action_permission() {
        let result =
            AccessRequest::for_principal(User::new(1).with_permission(Permission::PostRead))
                .performing_action(PostAction::Read)
                .check_action_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_action_permission() {
        let err = AccessRequest::for_principal(User::new(2).with_permission(Permission::PostRead))
            .performing_action(PostAction::Write)
            .check_action_permission()
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_all_action_permissions() {
        let result = AccessRequest::for_principal(
            User::new(1)
                .with_permission(Permission::PostRead)
                .with_permission(Permission::PostWrite),
        )
        .performing_action(PostAction::Write)
        .check_action_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_some_action_permissions() {
        let err = AccessRequest::for_principal(User::new(1).with_permission(Permission::PostRead))
            .performing_action(PostAction::Write)
            .check_action_permission()
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_any_action_permission() {
        let result =
            AccessRequest::for_principal(User::new(1).with_permission(Permission::PostWrite))
                .performing_action(PostAction::Write)
                .check_action_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_on_possessing_resource_permission() {
        let user_id = 1;

        let result =
            AccessRequest::for_principal(User::new(1).with_permission(Permission::PostWrite))
                .performing_action(PostAction::Write)
                .on_resource(Post::with_owner(user_id))
                .check_resource_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_resource_permission() {
        let user_id = 1;

        let err =
            AccessRequest::for_principal(User::new(user_id).with_permission(Permission::PostRead))
                .performing_action(PostAction::Write)
                .on_resource(Post::with_owner(user_id))
                .check_resource_permission()
                .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }
}
