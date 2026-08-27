use crate::{ActionRequest, Grant, Policy, PolicyDecision, RequestError, ResourceRequest, Subject};

pub trait Action {
    type Permission: Eq;

    fn required_permission(&self) -> Self::Permission;
}

pub trait HasPermission: Subject {
    type Permission: Eq;

    fn has_permission(&self, permission: &Self::Permission) -> bool;
}

pub trait HasPermissionOn<R>: Subject {
    type Permission: Eq;

    fn has_permission_on(&self, permission: &Self::Permission, resource: &R) -> bool;
}

pub struct RequirePermission;

impl<S, A, R> Policy<S, A, R> for RequirePermission
where
    S: HasPermission<Permission = A::Permission>,
    A: Action,
{
    fn evaluate(&self, subject: &S, action: &A, _: &R) -> PolicyDecision {
        if subject.has_permission(&action.required_permission()) {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<S, A> ActionRequest<S, A>
where
    S: HasPermission<Permission = A::Permission>,
    A: Action,
{
    pub fn check_action_permission(self) -> Result<Grant<S, A>, RequestError> {
        self.authorize(&RequirePermission)
    }
}

pub struct RequirePermissionOn;

impl<S, A, R> Policy<S, A, R> for RequirePermissionOn
where
    S: HasPermissionOn<R, Permission = A::Permission>,
    A: Action,
{
    fn evaluate(&self, subject: &S, action: &A, resource: &R) -> PolicyDecision {
        if subject.has_permission_on(&action.required_permission(), resource) {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }
}

impl<S, A, R> ResourceRequest<S, A, R>
where
    S: HasPermissionOn<R, Permission = A::Permission>,
    A: Action,
{
    pub fn check_resource_permission(self) -> Result<Grant<S, A, R>, RequestError> {
        self.authorize(&RequirePermissionOn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessRequest, test_utils::*};

    #[test]
    fn grants_access_on_possessing_action_permission() {
        let result = AccessRequest::for_subject(User::new(1).with_permission(Permission::PostRead))
            .performing_action(PostAction::Read)
            .check_action_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_action_permission() {
        let err = AccessRequest::for_subject(User::new(2).with_permission(Permission::PostRead))
            .performing_action(PostAction::Write)
            .check_action_permission()
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_all_action_permissions() {
        let result = AccessRequest::for_subject(
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
        let err = AccessRequest::for_subject(User::new(1).with_permission(Permission::PostRead))
            .performing_action(PostAction::Write)
            .check_action_permission()
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_any_action_permission() {
        let result =
            AccessRequest::for_subject(User::new(1).with_permission(Permission::PostWrite))
                .performing_action(PostAction::Write)
                .check_action_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_on_possessing_resource_permission() {
        let user_id = 1;

        let result =
            AccessRequest::for_subject(User::new(1).with_permission(Permission::PostWrite))
                .performing_action(PostAction::Write)
                .on_resource(Post::with_owner(user_id))
                .check_resource_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_resource_permission() {
        let user_id = 1;

        let err =
            AccessRequest::for_subject(User::new(user_id).with_permission(Permission::PostRead))
                .performing_action(PostAction::Write)
                .on_resource(Post::with_owner(user_id))
                .check_resource_permission()
                .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }
}
