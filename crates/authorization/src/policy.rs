pub trait Policy<S, R, A = ()> {
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PolicyDecision {
    #[default]
    Deny,
    Allow,
}

pub struct Allow;

impl<S, R, A> Policy<S, R, A> for Allow {
    fn evaluate(&self, _: &S, _: &R, _: &A) -> PolicyDecision {
        PolicyDecision::Allow
    }
}

pub struct Deny;

impl<S, R, A> Policy<S, R, A> for Deny {
    fn evaluate(&self, _: &S, _: &R, _: &A) -> PolicyDecision {
        PolicyDecision::Deny
    }
}

pub struct All<S, R, A> {
    policies: Vec<Box<dyn Policy<S, R, A>>>,
}

impl<S, R, A> All<S, R, A> {
    #[must_use]
    pub fn new(policies: Vec<Box<dyn Policy<S, R, A>>>) -> Self {
        Self { policies }
    }
}

impl<S, R, A> Policy<S, R, A> for All<S, R, A> {
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        if self.policies.is_empty() {
            return PolicyDecision::Deny;
        }

        if self
            .policies
            .iter()
            .all(|policy| policy.evaluate(subject, resource, action) == PolicyDecision::Allow)
        {
            return PolicyDecision::Allow;
        }

        PolicyDecision::Deny
    }
}

pub struct Any<S, R, A> {
    policies: Vec<Box<dyn Policy<S, R, A>>>,
}

impl<S, R, A> Any<S, R, A> {
    #[must_use]
    pub fn new(policies: Vec<Box<dyn Policy<S, R, A>>>) -> Self {
        Self { policies }
    }
}

impl<S, R, A> Policy<S, R, A> for Any<S, R, A> {
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        if self
            .policies
            .iter()
            .any(|policy| policy.evaluate(subject, resource, action) == PolicyDecision::Allow)
        {
            return PolicyDecision::Allow;
        }

        PolicyDecision::Deny
    }
}

pub struct FnPolicy<F>(F);

impl<F, S, R, A> Policy<S, R, A> for FnPolicy<F>
where
    F: Fn(&S, &R, &A) -> PolicyDecision,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        (self.0)(subject, resource, action)
    }
}

pub fn policy<F>(f: F) -> FnPolicy<F> {
    FnPolicy(f)
}
