use crate::{
    Grant, Policy, PolicyDecision, PrincipalRequest, RequestError,
    policy::{All, Any},
};

pub trait HasRole {
    type Role: Eq;

    fn has_role(&self, role: &Self::Role) -> bool;
}

pub struct RequireRole<R>(R);

impl<R> RequireRole<R> {
    pub const fn new(role: R) -> Self {
        Self(role)
    }
}

impl<P, R> Policy<PrincipalRequest<P>> for RequireRole<R>
where
    P: HasRole<Role = R>,
{
    fn evaluate(&self, request: &PrincipalRequest<P>) -> PolicyDecision {
        if request.principal.has_role(&self.0) {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<P> PrincipalRequest<P>
where
    P: HasRole,
{
    pub fn check_role(self, role: P::Role) -> Result<Grant<P>, RequestError> {
        self.authorize(&RequireRole::new(role))
    }

    pub fn check_all_roles(
        self,
        roles: impl IntoIterator<Item = P::Role>,
    ) -> Result<Grant<P>, RequestError> {
        let policy = All::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }

    pub fn check_any_role(
        self,
        roles: impl IntoIterator<Item = P::Role>,
    ) -> Result<Grant<P>, RequestError> {
        let policy = Any::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessRequest, test_utils::*};

    #[test]
    fn grants_access_on_posessing_role() {
        let result =
            AccessRequest::for_principal(User::new(1).with_role(Role::User)).check_role(Role::User);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_role() {
        let err = AccessRequest::for_principal(User::default())
            .check_role(Role::Admin)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_all_roles() {
        let result = AccessRequest::for_principal(
            User::new(1).with_role(Role::User).with_role(Role::Editor),
        )
        .check_all_roles([Role::User, Role::Editor]);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_some_roles() {
        let err = AccessRequest::for_principal(User::new(1).with_role(Role::User))
            .check_all_roles([Role::User, Role::Editor])
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_any_role() {
        let result = AccessRequest::for_principal(User::new(1).with_role(Role::User))
            .check_any_role([Role::User, Role::Editor]);

        assert!(result.is_ok());
    }
}
