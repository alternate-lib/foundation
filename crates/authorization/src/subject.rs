pub trait Subject {
    type Identity: Copy + Eq + 'static;

    fn identity(&self) -> Self::Identity;
}

impl Subject for () {
    type Identity = ();

    fn identity(&self) -> Self::Identity {}
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Authenticated<I>(I);

impl<I> Authenticated<I> {
    pub fn new(identity: I) -> Self {
        Self(identity)
    }
}

impl<I: Copy + Eq + 'static> Subject for Authenticated<I> {
    type Identity = I;

    fn identity(&self) -> Self::Identity {
        self.0
    }
}
