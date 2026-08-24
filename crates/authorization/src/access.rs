use std::marker::PhantomData;

use crate::{Policy, PolicyDecision, Subject};

pub struct AccessRequest<S, R, A = ()> {
    subject: S,
    resource: R,
    action: A,
}

impl<S: Subject, R> AccessRequest<S, R, ()> {
    pub fn new(subject: S, resource: R) -> AccessRequest<S, R> {
        Self {
            subject,
            resource,
            action: (),
        }
    }
}

impl<S, R> AccessRequest<S, R, ()> {
    #[must_use]
    pub fn with_action<A>(self, action: A) -> AccessRequest<S, R, A> {
        AccessRequest {
            subject: self.subject,
            resource: self.resource,
            action,
        }
    }
}

impl<S, R, A> AccessRequest<S, R, A> {
    pub fn authorize<P: Policy<S, R, A>>(
        self,
        policy: &P,
    ) -> Result<Authorized<R, A>, AccessRequestError> {
        match policy.evaluate(&self.subject, &self.resource, &self.action) {
            PolicyDecision::Permit => Ok(Authorized {
                resource: self.resource,
                _action: PhantomData,
            }),
            PolicyDecision::Deny => Err(AccessRequestError::Denied),
            PolicyDecision::NotApplicable => Err(AccessRequestError::NotApplicable),
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
pub enum AccessRequestError {
    #[error("access denied")]
    Denied,

    #[error("no applicable policy found")]
    NotApplicable,
}
