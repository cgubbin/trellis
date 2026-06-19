use crate::engine::EngineEvent;
use crate::state::{StateView, UserState};
use crate::Termination;

mod csv_file;
mod plot;
mod tracing;

use std::sync::{Arc, Mutex};

pub trait Observe<S: UserState>: Send + Sync {
    fn observe(&self, name: &'static str, state: StateView<'_, S>, event: &EngineEvent<S::Float>);
}

pub struct Observers<S> {
    inner: Vec<(Arc<Mutex<dyn Observe<S>>>, Frequency)>,
}

impl<S: UserState> Observers<S> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn attach(&mut self, observer: Arc<Mutex<dyn Observe<S>>>, frequency: Frequency) {
        self.inner.push((observer, frequency))
    }

    pub fn dispatch(
        &self,
        ident: &'static str,
        state: StateView<'_, S>,
        event: &EngineEvent<S::Float>,
    ) {
        for (obs, freq) in &self.inner {
            if freq.should_run(event, state.iteration()) {
                let obs = obs.lock().unwrap();
                obs.observe(ident, state.clone(), event);
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Frequency {
    Always,
    Every(usize),
    OnExit,
    Never,
}

impl Frequency {
    fn should_run<F>(&self, event: &EngineEvent<F>, iteration: usize) -> bool {
        match self {
            Self::Never => false,
            Self::Always => true,
            Self::OnExit => matches!(event, EngineEvent::Termination(_)),
            Self::Every(n) if iteration % n == 0 => true,
            Self::Every(_) => false,
        }
    }
}
