use crate::{
    ActionRequest, Grant, Policy, PolicyDecision, RequestError, ResourceRequest, Subject,
    SubjectRequest,
    policy::{All, Any},
};

pub trait HasRole: Subject {
    type Role: Eq;

    fn has_role(&self, role: &Self::Role) -> bool;
}

pub struct RequireRole<R>(R);

impl<R> RequireRole<R> {
    pub const fn new(role: R) -> Self {
        Self(role)
    }
}

impl<S, Re, A, Ro> Policy<S, Re, A> for RequireRole<Ro>
where
    S: HasRole<Role = Ro>,
{
    fn evaluate(&self, subject: &S, _: &Re, _: &A) -> PolicyDecision {
        if subject.has_role(&self.0) {
            return PolicyDecision::Permit;
        }

        PolicyDecision::Deny
    }
}

impl<S> SubjectRequest<S>
where
    S: HasRole,
{
    pub fn check_role(self, role: S::Role) -> Result<Grant<S>, RequestError> {
        self.authorize(&RequireRole::new(role))
    }

    pub fn check_all_roles(
        self,
        roles: impl IntoIterator<Item = S::Role>,
    ) -> Result<Grant<S>, RequestError> {
        let policy = All::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }

    pub fn check_any_role(
        self,
        roles: impl IntoIterator<Item = S::Role>,
    ) -> Result<Grant<S>, RequestError> {
        let policy = Any::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }
}

impl<S, A> ActionRequest<S, A>
where
    S: HasRole,
{
    pub fn check_role(self, role: S::Role) -> Result<Grant<S, A>, RequestError> {
        self.authorize(&RequireRole::new(role))
    }

    pub fn check_all_roles(
        self,
        roles: impl IntoIterator<Item = S::Role>,
    ) -> Result<Grant<S, A>, RequestError> {
        let policy = All::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }

    pub fn check_any_role(
        self,
        roles: impl IntoIterator<Item = S::Role>,
    ) -> Result<Grant<S, A>, RequestError> {
        let policy = Any::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }
}

impl<S, A, R> ResourceRequest<S, A, R>
where
    S: HasRole,
{
    pub fn check_role(self, role: S::Role) -> Result<Grant<S, A, R>, RequestError> {
        self.authorize(&RequireRole::new(role))
    }

    pub fn check_all_roles(
        self,
        roles: impl IntoIterator<Item = S::Role>,
    ) -> Result<Grant<S, A, R>, RequestError> {
        let policy = All::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }

    pub fn check_any_role(
        self,
        roles: impl IntoIterator<Item = S::Role>,
    ) -> Result<Grant<S, A, R>, RequestError> {
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
            AccessRequest::for_subject(User::new(1).with_role(Role::User)).check_role(Role::User);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_role() {
        let err = AccessRequest::for_subject(User::default())
            .check_role(Role::Admin)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_all_roles() {
        let result =
            AccessRequest::for_subject(User::new(1).with_role(Role::User).with_role(Role::Editor))
                .check_all_roles([Role::User, Role::Editor]);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_some_roles() {
        let err = AccessRequest::for_subject(User::new(1).with_role(Role::User))
            .check_all_roles([Role::User, Role::Editor])
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_posessing_any_role() {
        let result = AccessRequest::for_subject(User::new(1).with_role(Role::User))
            .check_any_role([Role::User, Role::Editor]);

        assert!(result.is_ok());
    }
}
