#[derive(Debug)]
pub struct Problem<P>(P);

impl<P> Problem<P> {
    pub(crate) fn new(inner: P) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> P {
        self.0
    }

    pub fn inner(&self) -> &P {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut P {
        &mut self.0
    }
}

impl<P> AsRef<P> for Problem<P> {
    fn as_ref(&self) -> &P {
        &self.0
    }
}

impl<P> AsMut<P> for Problem<P> {
    fn as_mut(&mut self) -> &mut P {
        &mut self.0
    }
}
