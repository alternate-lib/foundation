use std::ops::Deref;

use crate::Entity;

pub trait AggregateRoot: Entity {
    type Ev: Event;

    const TYPE: &'static str;

    fn version(&self) -> Version;

    fn drain_events(&mut self) -> Vec<Self::Ev>;
}

pub trait Event {
    fn event_type(&self) -> &'static str;
}

impl Event for () {
    fn event_type(&self) -> &'static str {
        ""
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Version {
    inner: u64,
    dirty: bool,
}

impl Version {
    #[must_use]
    pub fn incremented(self) -> Self {
        if self.dirty {
            self
        } else {
            Self {
                inner: self.inner + 1,
                dirty: true,
            }
        }
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn restore(version: u64) -> Self {
        Self {
            inner: version,
            dirty: false,
        }
    }
}

impl Deref for Version {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub struct EventList<E: Event>(Vec<E>);

impl<E: Event> Default for EventList<E> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<E: Event> EventList<E> {
    pub fn push(&mut self, event: E) {
        self.0.push(event);
    }

    pub fn drain(&mut self) -> Vec<E> {
        std::mem::take(&mut self.0)
    }
}
