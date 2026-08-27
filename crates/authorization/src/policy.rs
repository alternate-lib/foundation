use alternate_logic::{
    Conjunction, Disjunction, Logic, Negation, Predicate,
    combinator::{
        All as LogicAll, And as LogicAnd, Any as LogicAny, Not as LogicNot, Or as LogicOr,
    },
};

pub trait Policy<R> {
    fn evaluate(&self, request: &R) -> PolicyDecision;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PolicyDecision {
    Permit,
    #[default]
    Deny,
    NotApplicable,
}

struct PolicyLogic;

impl Logic for PolicyLogic {
    type Value = PolicyDecision;
}

impl Conjunction for PolicyLogic {
    fn and(left: PolicyDecision, right: PolicyDecision) -> PolicyDecision {
        match (left, right) {
            (PolicyDecision::Permit, PolicyDecision::Permit) => PolicyDecision::Permit,
            (PolicyDecision::Deny, _) | (_, PolicyDecision::Deny) => PolicyDecision::Deny,
            (PolicyDecision::NotApplicable, d) | (d, PolicyDecision::NotApplicable) => d,
        }
    }

    fn identity() -> PolicyDecision {
        PolicyDecision::NotApplicable
    }

    fn short_circuit(left: &PolicyDecision) -> Option<PolicyDecision> {
        match left {
            PolicyDecision::Deny => Some(PolicyDecision::Deny),
            _ => None,
        }
    }
}

impl Disjunction for PolicyLogic {
    fn or(left: PolicyDecision, right: PolicyDecision) -> PolicyDecision {
        match (left, right) {
            (PolicyDecision::Permit, _) | (_, PolicyDecision::Permit) => PolicyDecision::Permit,
            (PolicyDecision::Deny, PolicyDecision::Deny) => PolicyDecision::Deny,
            (PolicyDecision::NotApplicable, d) | (d, PolicyDecision::NotApplicable) => d,
        }
    }

    fn identity() -> PolicyDecision {
        PolicyDecision::NotApplicable
    }

    fn short_circuit(left: &PolicyDecision) -> Option<PolicyDecision> {
        match left {
            PolicyDecision::Permit => Some(PolicyDecision::Permit),
            _ => None,
        }
    }
}

impl Negation for PolicyLogic {
    fn not(value: PolicyDecision) -> PolicyDecision {
        match value {
            PolicyDecision::Permit => PolicyDecision::Deny,
            PolicyDecision::Deny => PolicyDecision::Permit,
            PolicyDecision::NotApplicable => PolicyDecision::NotApplicable,
        }
    }
}

struct PolicyContext<'a, R> {
    request: &'a R,
}

struct PolicyPredicate<P>(P);

impl<'a, R, P> Predicate<PolicyContext<'a, R>> for PolicyPredicate<P>
where
    P: Policy<R>,
{
    type Logic = PolicyLogic;

    fn evaluate(&self, context: &PolicyContext<'a, R>) -> PolicyDecision {
        self.0.evaluate(context.request)
    }
}

pub struct Permit;

impl<R> Policy<R> for Permit {
    fn evaluate(&self, _: &R) -> PolicyDecision {
        PolicyDecision::Permit
    }
}

pub struct Deny;

impl<R> Policy<R> for Deny {
    fn evaluate(&self, _: &R) -> PolicyDecision {
        PolicyDecision::Deny
    }
}

pub struct NotApplicable;

impl<R> Policy<R> for NotApplicable {
    fn evaluate(&self, _: &R) -> PolicyDecision {
        PolicyDecision::NotApplicable
    }
}

pub struct FnPolicy<F>(F);

impl<F> FnPolicy<F> {
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<R, F> Policy<R> for FnPolicy<F>
where
    F: Fn(&R) -> PolicyDecision,
{
    fn evaluate(&self, request: &R) -> PolicyDecision {
        (self.0)(request)
    }
}

pub struct And<L, R>(LogicAnd<PolicyPredicate<L>, PolicyPredicate<R>>);

impl<L, R> And<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(LogicAnd::new(PolicyPredicate(left), PolicyPredicate(right)))
    }
}

impl<Re, L, Ri> Policy<Re> for And<L, Ri>
where
    L: Policy<Re>,
    Ri: Policy<Re>,
{
    fn evaluate(&self, request: &Re) -> PolicyDecision {
        Predicate::evaluate(&self.0, &PolicyContext { request })
    }
}

pub struct Or<L, R>(LogicOr<PolicyPredicate<L>, PolicyPredicate<R>>);

impl<L, R> Or<L, R> {
    pub const fn new(left: L, right: R) -> Self {
        Self(LogicOr::new(PolicyPredicate(left), PolicyPredicate(right)))
    }
}

impl<Re, L, Ri> Policy<Re> for Or<L, Ri>
where
    L: Policy<Re>,
    Ri: Policy<Re>,
{
    fn evaluate(&self, request: &Re) -> PolicyDecision {
        Predicate::evaluate(&self.0, &PolicyContext { request })
    }
}

pub struct Not<P>(LogicNot<PolicyPredicate<P>>);

impl<P> Not<P> {
    pub const fn new(policy: P) -> Self {
        Self(LogicNot::new(PolicyPredicate(policy)))
    }
}

impl<R, P> Policy<R> for Not<P>
where
    P: Policy<R>,
{
    fn evaluate(&self, request: &R) -> PolicyDecision {
        Predicate::evaluate(&self.0, &PolicyContext { request })
    }
}

pub struct All<P>(LogicAll<PolicyPredicate<P>>);

impl<P> All<P> {
    pub const fn new() -> Self {
        Self(LogicAll::new())
    }

    #[must_use]
    pub fn with_policy(mut self, policy: P) -> Self {
        self.0 = self.0.with_predicate(PolicyPredicate(policy));
        self
    }
}

impl<P> Default for All<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> FromIterator<P> for All<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(PolicyPredicate).collect())
    }
}

impl<R, P> Policy<R> for All<P>
where
    P: Policy<R>,
{
    fn evaluate(&self, request: &R) -> PolicyDecision {
        Predicate::evaluate(&self.0, &PolicyContext { request })
    }
}

pub struct Any<P>(LogicAny<PolicyPredicate<P>>);

impl<P> Any<P> {
    pub const fn new() -> Self {
        Self(LogicAny::new())
    }

    #[must_use]
    pub fn with_policy(mut self, policy: P) -> Self {
        self.0 = self.0.with_predicate(PolicyPredicate(policy));
        self
    }
}

impl<P> Default for Any<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<P> FromIterator<P> for Any<P> {
    fn from_iter<T: IntoIterator<Item = P>>(iter: T) -> Self {
        Self(iter.into_iter().map(PolicyPredicate).collect())
    }
}

impl<R, P> Policy<R> for Any<P>
where
    P: Policy<R>,
{
    fn evaluate(&self, request: &R) -> PolicyDecision {
        Predicate::evaluate(&self.0, &PolicyContext { request })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decision(p: &impl Policy<()>) -> PolicyDecision {
        p.evaluate(&())
    }

    fn fn_policy(f: fn(&()) -> PolicyDecision) -> FnPolicy<fn(&()) -> PolicyDecision> {
        FnPolicy(f)
    }

    #[test]
    fn handles_conjunction() {
        assert_eq!(
            PolicyLogic::and(PolicyDecision::Permit, PolicyDecision::Permit),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::Permit, PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::Permit, PolicyDecision::NotApplicable),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::Deny, PolicyDecision::Permit),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::Deny, PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::Deny, PolicyDecision::NotApplicable),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::NotApplicable, PolicyDecision::Permit),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::NotApplicable, PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::and(PolicyDecision::NotApplicable, PolicyDecision::NotApplicable),
            PolicyDecision::NotApplicable
        );
    }

    #[test]
    fn handles_disjunction() {
        assert_eq!(
            PolicyLogic::or(PolicyDecision::Permit, PolicyDecision::Permit),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::Permit, PolicyDecision::Deny),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::Permit, PolicyDecision::NotApplicable),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::Deny, PolicyDecision::Permit),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::Deny, PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::Deny, PolicyDecision::NotApplicable),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::NotApplicable, PolicyDecision::Permit),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::NotApplicable, PolicyDecision::Deny),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::or(PolicyDecision::NotApplicable, PolicyDecision::NotApplicable),
            PolicyDecision::NotApplicable
        );
    }

    #[test]
    fn handles_negation() {
        assert_eq!(
            PolicyLogic::not(PolicyDecision::Permit),
            PolicyDecision::Deny
        );
        assert_eq!(
            PolicyLogic::not(PolicyDecision::Deny),
            PolicyDecision::Permit
        );
        assert_eq!(
            PolicyLogic::not(PolicyDecision::NotApplicable),
            PolicyDecision::NotApplicable
        );
    }

    #[test]
    fn short_circuits_conjunction_on_denying_policy() {
        let panic_if_evaluated = fn_policy(|()| panic!("right side of And must not be evaluated"));

        assert_eq!(
            decision(&And::new(Deny, panic_if_evaluated)),
            PolicyDecision::Deny
        );
    }

    #[test]
    fn short_circuits_disjunction_on_permitting_policy() {
        let panic_if_evaluated = fn_policy(|()| panic!("right side of Or must not be evaluated"));

        assert_eq!(
            decision(&Or::new(Permit, panic_if_evaluated)),
            PolicyDecision::Permit
        );
    }

    #[test]
    fn does_not_evalute_policies_on_empty_collections() {
        assert_eq!(
            decision(&All::<Permit>::new()),
            PolicyDecision::NotApplicable
        );
        assert_eq!(
            decision(&Any::<Permit>::new()),
            PolicyDecision::NotApplicable
        );
    }
}
