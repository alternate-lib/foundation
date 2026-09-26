use crate::{Policy, PolicyDecision};

pub struct AccessRequest;

impl AccessRequest {
    pub fn for_principal<P>(principal: P) -> PrincipalRequest<P> {
        PrincipalRequest { principal }
    }
}

#[derive(Debug)]
pub struct PrincipalRequest<P> {
    principal: P,
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

    #[must_use]
    pub fn using_resource<R>(self, resource: R) -> ResourceRequest<P, (), R> {
        ResourceRequest {
            principal: self.principal,
            action: (),
            resource,
        }
    }
}

impl<Pr> PrincipalRequest<Pr> {
    pub fn authorize<Po: Policy<Self>>(self, policy: &Po) -> Result<Grant<Self>, RequestError> {
        Grant::authorize(self, policy)
    }

    pub fn principal(&self) -> &Pr {
        &self.principal
    }
}

#[derive(Debug)]
pub struct ActionRequest<P, A> {
    principal: P,
    action: A,
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
    pub fn authorize<Po: Policy<Self>>(self, policy: &Po) -> Result<Grant<Self>, RequestError> {
        Grant::authorize(self, policy)
    }

    pub fn principal(&self) -> &Pr {
        &self.principal
    }

    pub fn action(&self) -> &A {
        &self.action
    }
}

#[derive(Debug)]
pub struct ResourceRequest<P, A, R> {
    principal: P,
    action: A,
    resource: R,
}

impl<Pr, A, R> ResourceRequest<Pr, A, R> {
    pub fn new(principal: Pr, action: A, resource: R) -> Self {
        Self {
            principal,
            action,
            resource,
        }
    }

    pub fn authorize<Po: Policy<Self>>(self, policy: &Po) -> Result<Grant<Self>, RequestError> {
        Grant::authorize(self, policy)
    }

    pub fn principal(&self) -> &Pr {
        &self.principal
    }

    pub fn action(&self) -> &A {
        &self.action
    }

    pub fn resource(&self) -> &R {
        &self.resource
    }
}

#[derive(Debug)]
pub struct Grant<R>(R);

impl<R> Grant<R> {
    pub fn authorize<P: Policy<R>>(request: R, policy: &P) -> Result<Self, RequestError> {
        match policy.evaluate(&request) {
            PolicyDecision::Permit => Ok(Self(request)),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }

    pub fn request(&self) -> &R {
        &self.0
    }

    pub fn into_request(self) -> R {
        self.0
    }
}

impl<P> Grant<PrincipalRequest<P>> {
    pub fn principal(&self) -> &P {
        self.0.principal()
    }
}

impl<P, A> Grant<ActionRequest<P, A>> {
    pub fn principal(&self) -> &P {
        self.0.principal()
    }

    pub fn action(&self) -> &A {
        self.0.action()
    }
}

impl<P, A, R> Grant<ResourceRequest<P, A, R>> {
    pub fn principal(&self) -> &P {
        self.0.principal()
    }

    pub fn action(&self) -> &A {
        self.0.action()
    }

    pub fn into_resource(self) -> R {
        self.0.resource
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
        let result = AccessRequest::for_principal(User::default()).authorize(&Permit);

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_to_action_for_permitting_policy() {
        let result = AccessRequest::for_principal(User::default())
            .performing_action("read")
            .authorize(&Permit);

        assert!(result.is_ok());
    }

    #[test]
    fn denies_access_for_denying_policy() {
        let err = AccessRequest::for_principal(User::default())
            .authorize(&Deny)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn denies_access_for_not_applicable_policy() {
        let result = AccessRequest::for_principal(User::default())
            .authorize(&NotApplicable)
            .unwrap_err();

        assert!(matches!(result, RequestError::NotApplicable));
    }

    #[test]
    fn authorizes_custom_requests() {
        #[derive(Debug, PartialEq, Eq)]
        struct CustomRequest(u64);

        let grant = Grant::authorize(CustomRequest(42), &Permit).unwrap();

        assert_eq!(grant.request(), &CustomRequest(42));
        assert_eq!(grant.into_request(), CustomRequest(42));
    }
}
