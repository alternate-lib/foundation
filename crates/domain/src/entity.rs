use std::hash::Hash;

pub trait Entity: Eq + Hash {
    type Id: Eq + Hash;
    type Snapshot;

    fn id(&self) -> Self::Id;

    fn snapshot(&self) -> Self::Snapshot;

    fn restore(snapshot: Self::Snapshot) -> Self;
}

#[macro_export]
macro_rules! impl_entity {
    ($type:ty, $snapshot:ty) => {
        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                $crate::Entity::id(self) == $crate::Entity::id(other)
            }
        }

        impl Eq for $type {}

        impl std::hash::Hash for $type {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                $crate::Entity::id(self).hash(state);
            }
        }

        impl From<$snapshot> for $type {
            fn from(value: $snapshot) -> Self {
                $crate::Entity::restore(value)
            }
        }

        impl From<&$type> for $snapshot {
            fn from(value: &$type) -> Self {
                $crate::Entity::snapshot(value)
            }
        }
    };
}
