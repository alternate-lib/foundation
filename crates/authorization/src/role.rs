use crate::{
    AccessRequest, AccessRequestError, Authorized, Policy, PolicyDecision, Subject,
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

impl<S, Re, A, Ro> AccessRequest<S, Re, A>
where
    S: HasRole<Role = Ro>,
{
    pub fn check_role(self, role: Ro) -> Result<Authorized<Re, A>, AccessRequestError> {
        self.authorize(&RequireRole::new(role))
    }

    pub fn check_all_roles(
        self,
        roles: impl IntoIterator<Item = Ro>,
    ) -> Result<Authorized<Re, A>, AccessRequestError> {
        let policy = All::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }

    pub fn check_any_role(
        self,
        roles: impl IntoIterator<Item = Ro>,
    ) -> Result<Authorized<Re, A>, AccessRequestError> {
        let policy = Any::from_iter(roles.into_iter().map(RequireRole::new));

        self.authorize(&policy)
    }
}
