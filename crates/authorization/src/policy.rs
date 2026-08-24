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

impl PolicyDecision {
    fn and(self, other: Self) -> Self {
        if self == Self::Allow && other == Self::Allow {
            Self::Allow
        } else {
            Self::Deny
        }
    }

    fn or(self, other: Self) -> Self {
        if self == Self::Allow || other == Self::Allow {
            Self::Allow
        } else {
            Self::Deny
        }
    }

    fn not(self) -> Self {
        match self {
            Self::Allow => Self::Deny,
            Self::Deny => Self::Allow,
        }
    }
}

pub struct And<L, R>(L, R);

impl<L, R> And<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<S, R, A, L, P> Policy<S, R, A> for And<L, P>
where
    L: Policy<S, R, A>,
    P: Policy<S, R, A>,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        let left = self.0.evaluate(subject, resource, action);

        if left == PolicyDecision::Deny {
            return PolicyDecision::Deny;
        }

        left.and(self.1.evaluate(subject, resource, action))
    }
}

pub struct Or<L, R>(L, R);

impl<L, R> Or<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(left, right)
    }
}

impl<S, R, A, L, P> Policy<S, R, A> for Or<L, P>
where
    L: Policy<S, R, A>,
    P: Policy<S, R, A>,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        let left = self.0.evaluate(subject, resource, action);

        if left == PolicyDecision::Allow {
            return PolicyDecision::Allow;
        }

        left.or(self.1.evaluate(subject, resource, action))
    }
}

pub struct Not<P>(P);

impl<P> Not<P> {
    pub const fn new(policy: P) -> Self {
        Self(policy)
    }
}

impl<S, R, A, P> Policy<S, R, A> for Not<P>
where
    P: Policy<S, R, A>,
{
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision {
        self.0.evaluate(subject, resource, action).not()
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

        if self.0.iter().all(|policy| {
            PolicyDecision::Allow.and(policy.evaluate(subject, resource, action))
                == PolicyDecision::Allow
        }) {
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
        if self.0.iter().any(|policy| {
            PolicyDecision::Deny.or(policy.evaluate(subject, resource, action))
                == PolicyDecision::Allow
        }) {
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
