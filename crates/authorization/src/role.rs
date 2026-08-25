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
