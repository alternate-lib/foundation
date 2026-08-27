use crate::{Policy, PolicyDecision};

pub struct AccessRequest;

impl AccessRequest {
    pub fn for_subject<S>(subject: S) -> SubjectRequest<S> {
        SubjectRequest { subject }
    }
}

pub struct SubjectRequest<S> {
    subject: S,
}

impl<S> SubjectRequest<S> {
    pub fn new(subject: S) -> Self {
        Self { subject }
    }

    #[must_use]
    pub fn performing_action<A>(self, action: A) -> ActionRequest<S, A> {
        ActionRequest {
            subject: self.subject,
            action,
        }
    }
}

impl<S> SubjectRequest<S> {
    pub fn authorize<P>(self, policy: &P) -> Result<Grant<S>, RequestError>
    where
        P: Policy<S, (), ()>,
    {
        match policy.evaluate(&self.subject, &(), &()) {
            PolicyDecision::Permit => Ok(Grant {
                subject: self.subject,
                action: (),
                resource: (),
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

pub struct ActionRequest<S, A> {
    subject: S,
    action: A,
}

impl<S, A> ActionRequest<S, A> {
    pub fn new(subject: S, action: A) -> Self {
        Self { subject, action }
    }

    #[must_use]
    pub fn on_resource<R>(self, resource: R) -> ResourceRequest<S, A, R> {
        ResourceRequest {
            subject: self.subject,
            action: self.action,
            resource,
        }
    }
}

impl<S, A> ActionRequest<S, A> {
    pub fn authorize<P>(self, policy: &P) -> Result<Grant<S, A>, RequestError>
    where
        P: Policy<S, A, ()>,
    {
        match policy.evaluate(&self.subject, &self.action, &()) {
            PolicyDecision::Permit => Ok(Grant {
                subject: self.subject,
                action: self.action,
                resource: (),
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

pub struct ResourceRequest<S, A, R> {
    subject: S,
    action: A,
    resource: R,
}

impl<S, A, R> ResourceRequest<S, A, R> {
    pub fn new(subject: S, action: A, resource: R) -> Self {
        Self {
            subject,
            action,
            resource,
        }
    }

    pub fn authorize<P>(self, policy: &P) -> Result<Grant<S, A, R>, RequestError>
    where
        P: Policy<S, A, R>,
    {
        match policy.evaluate(&self.subject, &self.action, &self.resource) {
            PolicyDecision::Permit => Ok(Grant {
                subject: self.subject,
                action: self.action,
                resource: self.resource,
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

#[derive(Debug)]
pub struct Grant<S, A = (), R = ()> {
    subject: S,
    action: A,
    resource: R,
}

impl<S, A, R> Grant<S, A, R> {
    pub fn subject(&self) -> &S {
        &self.subject
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
    fn grants_access_to_subject_for_permitting_policy() {
        let result = AccessRequest::for_subject(User::new(1)).authorize(&Permit);

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_to_action_for_permitting_policy() {
        let result = AccessRequest::for_subject(User::new(1))
            .performing_action("read")
            .authorize(&Permit);

        assert!(result.is_ok());
    }

    #[test]
    fn grants_access_to_resource_for_permitting_policy() {
        let resource = "document";

        let grant = AccessRequest::for_subject(User::new(1))
            .performing_action("read")
            .on_resource(resource)
            .authorize(&Permit)
            .expect("permit must grant access");

        assert_eq!(grant.into_resource(), resource);
    }

    #[test]
    fn denies_access_for_denying_policy() {
        let err = AccessRequest::for_subject(User::new(1))
            .authorize(&Deny)
            .unwrap_err();

        assert!(matches!(err, RequestError::Denied));
    }

    #[test]
    fn denies_access_for_not_applicable_policy() {
        let result = AccessRequest::for_subject(User::new(1))
            .authorize(&NotApplicable)
            .unwrap_err();

        assert!(matches!(result, RequestError::NotApplicable));
    }
}
