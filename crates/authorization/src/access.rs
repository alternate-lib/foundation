use std::marker::PhantomData;

use crate::{Policy, PolicyDecision, Subject};

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
    pub fn authorize<P>(self, policy: &P) -> Result<(), RequestError>
    where
        P: Policy<S, (), ()>,
    {
        match policy.evaluate(&self.subject, &(), &()) {
            PolicyDecision::Permit => Ok(()),
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
    pub fn authorize<P>(self, policy: &P) -> Result<(), RequestError>
    where
        P: Policy<S, (), A>,
    {
        match policy.evaluate(&self.subject, &(), &self.action) {
            PolicyDecision::Permit => Ok(()),
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

    pub fn authorize<P>(self, policy: &P) -> Result<Authorized<R, A>, RequestError>
    where
        P: Policy<S, R, A>,
    {
        match policy.evaluate(&self.subject, &self.resource, &self.action) {
            PolicyDecision::Permit => Ok(Authorized {
                resource: self.resource,
                _action: PhantomData,
            }),
            PolicyDecision::Deny => Err(RequestError::Denied),
            PolicyDecision::NotApplicable => Err(RequestError::NotApplicable),
        }
    }
}

pub trait ReadAccess {}

impl ReadAccess for () {}

pub trait WriteAccess: ReadAccess {}

pub struct Authorized<R, A> {
    resource: R,
    _action: PhantomData<A>,
}

impl<R, A> Authorized<R, A> {
    pub fn into_inner(self) -> R {
        self.resource
    }
}

impl<R, A: WriteAccess> Authorized<R, A> {
    pub fn with_edit<E>(&mut self, f: impl FnOnce(&mut R) -> Result<(), E>) -> Result<(), E> {
        f(&mut self.resource)
    }
}

impl<R, A: ReadAccess> AsRef<R> for Authorized<R, A> {
    fn as_ref(&self) -> &R {
        &self.resource
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("access denied")]
    Denied,

    #[error("no applicable policy found")]
    NotApplicable,
}
