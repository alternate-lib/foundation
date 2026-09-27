use std::collections::{BTreeSet, HashSet};

pub trait AsView {
    type View: ?Sized;

    fn as_view(&self) -> &Self::View;
}

impl AsView for String {
    type View = str;

    fn as_view(&self) -> &Self::View {
        self.as_str()
    }
}

impl<T> AsView for Vec<T> {
    type View = [T];

    fn as_view(&self) -> &Self::View {
        self.as_slice()
    }
}

impl<T, S> AsView for HashSet<T, S> {
    type View = Self;

    fn as_view(&self) -> &Self::View {
        self
    }
}

impl<T> AsView for BTreeSet<T> {
    type View = Self;

    fn as_view(&self) -> &Self::View {
        self
    }
}

macro_rules! identity_view {
    ($($ty:ty),* $(,)?) => {
        $(impl AsView for $ty {
            type View = Self;

            fn as_view(&self) -> &Self::View {
                self
            }
        })*
    };
}

identity_view!(
    bool, char, f32, f64, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
);

#[cfg(feature = "uuid")]
identity_view!(uuid::Uuid);

#[cfg(feature = "jiff")]
identity_view!(jiff::Timestamp);
