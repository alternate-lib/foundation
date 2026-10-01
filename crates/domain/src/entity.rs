use std::{error::Error, fmt, hash::Hash};

pub trait Entity: Sized {
    type Id: Copy + Eq + Hash;
    type Snapshot;

    fn id(&self) -> Self::Id;

    fn snapshot(&self) -> Self::Snapshot;

    fn restore(snapshot: Self::Snapshot) -> Result<Self, EntityRestoreError>;
}

#[derive(Debug)]
pub struct EntityRestoreError {
    entity: &'static str,
    field: &'static str,
    source: Box<dyn Error + 'static>,
}

impl EntityRestoreError {
    pub fn new(entity: &'static str, field: &'static str, source: impl Error + 'static) -> Self {
        Self {
            entity,
            field,
            source: Box::new(source),
        }
    }

    pub fn entity(&self) -> &'static str {
        self.entity
    }

    pub fn field(&self) -> &'static str {
        self.field
    }
}

impl fmt::Display for EntityRestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "failed to restore {}.{}", self.entity, self.field)
    }
}

impl Error for EntityRestoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}
