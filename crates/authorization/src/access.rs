use crate::{Policy, PolicyDecision};

pub trait Subject {
    type Identity: Copy + Eq + 'static;

    fn identity(&self) -> Self::Identity;
}

pub struct AccessRequest;

impl AccessRequest {
    pub fn for_subject<S: Subject>(subject: S) -> SubjectRequest<S> {
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

impl<S: Subject> SubjectRequest<S> {
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

impl<S: Subject, A> ActionRequest<S, A> {
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

impl<S: Subject, A, R> ResourceRequest<S, A, R> {
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
