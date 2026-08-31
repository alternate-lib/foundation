use std::marker::PhantomData;

use crate::{
    Action, Grant, Grants, Permits, Policy, PolicyDecision, RequestError, ResourceRequest, RoleSet,
    permission::{RequireDirectPermission, RequirePermissionViaRole},
    policy::{And, Or},
};

pub trait Relates<T, Kind = ()> {
    fn relates(&self, target: &T) -> bool;
}

pub struct RequireRelation<Kind = ()>(PhantomData<Kind>);

impl<Kind> RequireRelation<Kind> {
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<K> Default for RequireRelation<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P, A, R, Kind> Policy<ResourceRequest<P, A, R>> for RequireRelation<Kind>
where
    P: Relates<R, Kind>,
{
    fn evaluate(&self, request: &ResourceRequest<P, A, R>) -> PolicyDecision {
        if request.principal.relates(&request.resource) {
            PolicyDecision::Permit
        } else {
            PolicyDecision::Deny
        }
    }
}

impl<P, A, R> ResourceRequest<P, A, R> {
    pub fn check_relation<Kind>(self) -> Result<Grant<P, A, R>, RequestError>
    where
        P: Relates<R, Kind>,
    {
        self.authorize(&RequireRelation::<Kind>::new())
    }
}

impl<P, A, R> ResourceRequest<P, A, R>
where
    P: RoleSet + Permits<A::Permission>,
    P::Role: Grants<A::Permission>,
    A: Action,
{
    pub fn check_relation_and_permission<Kind>(self) -> Result<Grant<P, A, R>, RequestError>
    where
        P: Relates<R, Kind>,
    {
        self.authorize(&And::new(
            RequireRelation::<Kind>::new(),
            Or::new(RequireDirectPermission, RequirePermissionViaRole),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AccessRequest, RequestError, test_utils::*};

    #[test]
    fn grants_access_on_possessing_relation() {
        let user = User::default();
        let user_id = user.id();

        let result = AccessRequest::for_principal(user)
            .using_resource(Post::with_owner(user_id))
            .check_relation();

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_on_missing_relation() {
        let err = AccessRequest::for_principal(User::new(1))
            .using_resource(Post::with_owner(2))
            .check_relation()
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn grants_access_on_possessing_relation_and_permission() {
        let user = User::default().with_permission(Permission::PostRead);
        let user_id = user.id();

        let result = AccessRequest::for_principal(user)
            .performing_action(PostAction::Read)
            .on_resource(Post::with_owner(user_id))
            .check_relation_and_permission();

        assert!(result.is_ok());
    }
}
