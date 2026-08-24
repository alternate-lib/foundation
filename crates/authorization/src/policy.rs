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

pub struct All<P>(Vec<P>);

impl<P> All<P> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn with_policy(mut self, policy: P) -> Self {
        self.0.push(policy);
        self
    }
}

impl<P> FromIterator<P> for All<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<P> Default for All<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, R, A, P> Policy<S, R, A> for All<P>
where
    P: Policy<S, R, A>,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        if self.0.is_empty() {
            return PolicyDecision::Deny;
        }

        if self
            .0
            .iter()
            .all(|policy| policy.evaluate(subject, resource, action) == PolicyDecision::Allow)
        {
            return PolicyDecision::Allow;
        }

        PolicyDecision::Deny
    }
}

pub struct Any<P>(Vec<P>);

impl<P> Any<P> {
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[must_use]
    pub fn with_policy(mut self, policy: P) -> Self {
        self.0.push(policy);
        self
    }
}

impl<P> FromIterator<P> for Any<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<P> Default for Any<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, R, A, P> Policy<S, R, A> for Any<P>
where
    P: Policy<S, R, A>,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        if self
            .0
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
