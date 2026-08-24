pub trait Subject {
    type Identity: Copy + Eq + 'static;

    fn identity(&self) -> Self::Identity;
}

pub trait Policy<S, R, A = ()> {
    fn evaluate(&self, subject: &S, resource: &R, action: &A) -> PolicyDecision;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PolicyDecision {
    #[default]
    Deny,
    Allow,
}
