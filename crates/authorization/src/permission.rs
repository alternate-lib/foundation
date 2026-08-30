use crate::{
    ActionRequest, Grant, ImpliedRoles as _, Policy, PolicyDecision, RequestError, RoleSet,
    policy::Or,
};

pub trait Action {
    type Permission;

    fn required_permission(&self) -> Self::Permission;
}

pub trait Grants<P> {
    fn grants(&self, permission: &P) -> bool;
}

pub trait Permits<P> {
    fn permits(&self, permission: &P) -> bool;
}

pub struct RequireDirectPermission;

impl<P, A> Policy<ActionRequest<P, A>> for RequireDirectPermission
where
    P: Permits<A::Permission>,
    A: Action,
{
    fn evaluate(&self, request: &ActionRequest<P, A>) -> PolicyDecision {
        if request
            .principal
            .permits(&request.action.required_permission())
        {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

pub struct RequirePermissionViaRole;

impl<P, A> Policy<ActionRequest<P, A>> for RequirePermissionViaRole
where
    P: RoleSet,
    <P as RoleSet>::Role: Grants<A::Permission>,
    A: Action,
{
    fn evaluate(&self, request: &ActionRequest<P, A>) -> PolicyDecision {
        if request.principal.roles().any(|assigned| {
            assigned
                .implied_roles()
                .any(|role| role.grants(&request.action.required_permission()))
        }) {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<P, A> ActionRequest<P, A>
where
    P: RoleSet + Permits<A::Permission>,
    <P as RoleSet>::Role: Grants<A::Permission>,
    A: Action,
{
    pub fn check_permission(self) -> Result<Grant<P, A>, RequestError> {
        self.authorize(&Or::new(RequireDirectPermission, RequirePermissionViaRole))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessRequest, test_utils::*};

    #[test]
    fn grants_access_on_possessing_direct_permission() {
        let result =
            AccessRequest::for_principal(User::default().with_permission(Permission::PostRead))
                .performing_action(PostAction::Read)
                .authorize(&RequireDirectPermission);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_direct_permission() {
        let err =
            AccessRequest::for_principal(User::default().with_permission(Permission::PostRead))
                .performing_action(PostAction::Write)
                .authorize(&RequireDirectPermission)
                .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_possessing_role_permission() {
        let result = AccessRequest::for_principal(User::default().with_role(Role::Editor))
            .performing_action(PostAction::Write)
            .authorize(&RequirePermissionViaRole);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_role_permission() {
        let err = AccessRequest::for_principal(User::default().with_role(Role::User))
            .performing_action(PostAction::Write)
            .authorize(&RequirePermissionViaRole)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn ignores_direct_permission_when_checking_role_permission() {
        let err =
            AccessRequest::for_principal(User::default().with_permission(Permission::PostWrite))
                .performing_action(PostAction::Write)
                .authorize(&RequirePermissionViaRole)
                .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn ignores_role_permission_when_checking_direct_permission() {
        let err = AccessRequest::for_principal(User::default().with_role(Role::Admin))
            .performing_action(PostAction::Write)
            .authorize(&RequireDirectPermission)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_via_direct_permission_only() {
        let result =
            AccessRequest::for_principal(User::default().with_permission(Permission::PostWrite))
                .performing_action(PostAction::Write)
                .check_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_via_role_permission_only() {
        let result = AccessRequest::for_principal(User::default().with_role(Role::Editor))
            .performing_action(PostAction::Write)
            .check_permission();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_direct_and_role_permissions() {
        let err = AccessRequest::for_principal(
            User::default()
                .with_role(Role::User)
                .with_permission(Permission::PostRead),
        )
        .performing_action(PostAction::Write)
        .check_permission()
        .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }
}
