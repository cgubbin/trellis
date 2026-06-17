use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct Problem<P> {
    inner: P,
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

// impl<P> Problem<P> {
//     pub(crate) fn new(inner: P) -> Self {
//         Self(inner)
//     }

//     pub fn into_inner(self) -> P {
//         self.0
//     }

//     pub fn inner(&self) -> &P {
//         &self.0
//     }

//     pub fn inner_mut(&mut self) -> &mut P {
//         &mut self.0
//     }
// }

// impl<P> AsRef<P> for Problem<P> {
//     fn as_ref(&self) -> &P {
//         &self.0
//     }
// }

// impl<P> AsMut<P> for Problem<P> {
//     fn as_mut(&mut self) -> &mut P {
//         &mut self.0
//     }
// }
