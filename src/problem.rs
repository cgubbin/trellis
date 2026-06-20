use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Problem<P> {
    pub inner: P,
}

impl<P> Problem<P> {
    pub(crate) fn new(inner: P) -> Self {
        Self { inner }
    }
}

impl<P> Deref for Problem<P> {
    type Target = P;

    fn deref(&self) -> &P {
        &self.inner
    }
}

impl<P> DerefMut for Problem<P> {
    fn deref_mut(&mut self) -> &mut P {
        &mut self.inner
    }
}
