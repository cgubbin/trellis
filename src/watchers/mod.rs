use std::sync::{Arc, Mutex};

// #[cfg(feature = "writing")]
// mod file;
//
// #[cfg(feature = "writing")]
// pub use file::FileWriter;
//
// #[cfg(feature = "plotting")]
// mod plot;
// #[cfg(feature = "plotting")]
// pub use plot::PlotGenerator;
//
mod tracing;

pub enum Target {
    Param,
    Measure,
}

#[derive(Copy, Clone)]
pub enum Stage {
    Initialisation,
    WrapUp,
    Iteration,
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub(crate) struct ObserverVec<S>(Vec<(Arc<Mutex<dyn Observer<S>>>, Frequency)>);

impl<S> ObserverVec<S> {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl<S> Default for ObserverVec<S> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<S> ObserverVec<S> {
    pub(crate) fn as_slice(&self) -> ObserverSlice<'_, S> {
        ObserverSlice(&self.0[..])
    }
}

#[allow(clippy::type_complexity)]
pub(crate) struct ObserverSlice<'a, S>(&'a [(Arc<Mutex<dyn Observer<S>>>, Frequency)]);

pub trait Observer<S>: Send + Sync {
    fn observe(&self, ident: &'static str, subject: &S, stage: Stage);
}

pub trait Observable<S> {
    type Observer;
    fn update(&self, ident: &'static str, subject: &S, stage: Stage);
    fn attach(&mut self, observer: Self::Observer, frequency: Frequency);
    fn detach(&mut self, observer: Self::Observer);
}

#[derive(Clone)]
pub(crate) struct Subject<D> {
    pub(crate) data: D,
    pub(crate) observers: ObserverVec<D>,
}

impl<S> Observable<S> for ObserverVec<S> {
    type Observer = Arc<Mutex<dyn Observer<S>>>;
    fn update(&self, ident: &'static str, subject: &S, stage: Stage) {
        self.0
            .iter()
            .map(|o| o.0.lock().unwrap())
            .for_each(|o| o.observe(ident, subject, stage));
    }
    fn attach(&mut self, observer: Self::Observer, frequency: Frequency) {
        self.0.push((observer, frequency));
    }
    fn detach(&mut self, observer: Self::Observer) {
        self.0.retain(|f| !Arc::ptr_eq(&f.0, &observer));
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObservationError {
    #[error("error in writer")]
    Writer(Box<dyn std::error::Error + 'static>), // We don't wrap the actual error, as we don't want to import the deps unless requested
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
// How often the observations should take place
pub enum Frequency {
    // An observer that never observes
    #[default]
    Never,
    // Observations occur on every iteration
    Always,
    // Observations occur on every nth iteration
    Every(usize),
    // The observer runs during the wrap up stage only
    OnExit,
}
