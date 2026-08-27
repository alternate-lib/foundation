use crate::{Policy, PolicyDecision};

pub struct AccessRequest;

impl AccessRequest {
    pub fn for_principal<P>(principal: P) -> PrincipalRequest<P> {
        PrincipalRequest { principal }
    }
}

pub struct PrincipalRequest<P> {
    pub(crate) principal: P,
}

impl<P> PrincipalRequest<P> {
    pub fn new(principal: P) -> Self {
        Self { principal }
    }

    #[must_use]
    pub fn performing_action<A>(self, action: A) -> ActionRequest<P, A> {
        ActionRequest {
            principal: self.principal,
            action,
        }
    }
}

impl<Pr> PrincipalRequest<Pr> {
    pub fn authorize<Po>(self, policy: &Po) -> Result<Grant<Pr>, RequestError>
    where
        Po: Policy<Self>,
    {
        match policy.evaluate(&self) {
            PolicyDecision::Permit => Ok(Grant {
                principal: self.principal,
                action: (),
                resource: (),
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

pub struct ActionRequest<P, A> {
    pub(crate) principal: P,
    pub(crate) action: A,
}

impl<P, A> ActionRequest<P, A> {
    pub fn new(principal: P, action: A) -> Self {
        Self { principal, action }
    }

    #[must_use]
    pub fn on_resource<R>(self, resource: R) -> ResourceRequest<P, A, R> {
        ResourceRequest {
            principal: self.principal,
            action: self.action,
            resource,
        }
    }
}

impl<Pr, A> ActionRequest<Pr, A> {
    pub fn authorize<Po>(self, policy: &Po) -> Result<Grant<Pr, A>, RequestError>
    where
        Po: Policy<Self>,
    {
        let decision = policy.evaluate(&self);

        match decision {
            PolicyDecision::Permit => Ok(Grant {
                principal: self.principal,
                action: self.action,
                resource: (),
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

pub struct ResourceRequest<P, A, R> {
    pub(crate) principal: P,
    pub(crate) action: A,
    pub(crate) resource: R,
}

impl<Pr, A, R> ResourceRequest<Pr, A, R> {
    pub fn new(principal: Pr, action: A, resource: R) -> Self {
        Self {
            principal,
            action,
            resource,
        }
    }

    pub fn authorize<Po>(self, policy: &Po) -> Result<Grant<Pr, A, R>, RequestError>
    where
        Po: Policy<Self>,
    {
        let decision = policy.evaluate(&self);

        match decision {
            PolicyDecision::Permit => Ok(Grant {
                principal: self.principal,
                action: self.action,
                resource: self.resource,
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

#[derive(Debug)]
pub struct Grant<P, A = (), R = ()> {
    principal: P,
    action: A,
    resource: R,
}

impl<P, A, R> Grant<P, A, R> {
    pub fn principal(&self) -> &P {
        &self.principal
    }

    pub fn action(&self) -> &A {
        &self.action
    }

    pub fn into_resource(self) -> R {
        self.resource
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("access denied")]
    Denied,

    #[error("no applicable policy found")]
    NotApplicable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        policy::{Deny, NotApplicable, Permit},
        test_utils::*,
    };

    #[test]
    fn grants_access_to_principal_for_permitting_policy() {
        let result = AccessRequest::for_principal(User::new(1)).authorize(&Permit);

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_to_action_for_permitting_policy() {
        let result = AccessRequest::for_principal(User::new(1))
            .performing_action("read")
            .authorize(&Permit);

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_to_resource_for_permitting_policy() {
        let resource = "document";

        let grant = AccessRequest::for_principal(User::new(1))
            .performing_action("read")
            .on_resource(resource)
            .authorize(&Permit)
            .expect("permit must grant access");

        assert_eq!(grant.into_resource(), resource);
    }

    #[test]
    fn denies_access_for_denying_policy() {
        let err = AccessRequest::for_principal(User::new(1))
            .authorize(&Deny)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn denies_access_for_not_applicable_policy() {
        let result = AccessRequest::for_principal(User::new(1))
            .authorize(&NotApplicable)
            .unwrap_err();

        assert!(matches!(result, RequestError::NotApplicable));
    }
}
