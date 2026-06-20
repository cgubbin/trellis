#![allow(clippy::type_complexity)]

use crate::engine::EngineSignal;
use crate::state::{StateView, UserState};

use std::sync::{Arc, Mutex};

mod csv_file;
mod failure;
mod metrics;
mod plot;
mod sampler;
mod tracing;

pub use csv_file::CsvProgressWriter;
pub use plot::PlotObserver;
pub use tracing::Tracer;

/// Core observer trait for the engine event system.
///
/// Observers receive a stream of [`EngineSignal`] events during execution
/// along with a read-only [`StateView`].
///
/// ### Design
/// - Uses `&self` to support shared observers (`Arc`)
/// - State mutation must use interior mutability if required
/// - Must be object-safe to allow dynamic dispatch
pub trait Observe<S: UserState>: Send + Sync {
    fn observe(&self, name: &'static str, state: StateView<'_, S>, event: &EngineSignal<S::Float>);
}

/// Blanket implementation for shared observers.
///
/// This allows `Arc<T>` to be used directly as an observer.
impl<S, T> Observe<S> for Arc<T>
where
    S: UserState,
    T: Observe<S> + ?Sized,
{
    fn observe(&self, name: &'static str, state: StateView<'_, S>, event: &EngineSignal<S::Float>) {
        (**self).observe(name, state, event)
    }
}

/// Frequency control for observer execution.
///
/// Allows observers to be:
/// - always active
/// - sampled at intervals
/// - triggered only on termination
/// - disabled entirely
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Frequency {
    Always,
    Every(usize),
    OnExit,
    Never,
}

impl Frequency {
    fn should_run<F>(&self, event: &EngineSignal<F>, iteration: usize) -> bool {
        match self {
            Self::Never => false,
            Self::Always => true,
            Self::OnExit => matches!(event, EngineSignal::Termination(_)),
            Self::Every(n) => *n != 0 && iteration.is_multiple_of(*n),
        }
    }
}

/// Container for all registered observers.
///
/// Observers are stored as `(observer, frequency)` pairs and dispatched
/// during engine execution.
pub struct Observers<S> {
    inner: Vec<(Arc<Mutex<dyn Observe<S>>>, Frequency)>,
}

impl<S: UserState> Observers<S> {
    /// Create an empty observer set.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// Attach a new observer with a frequency policy.
    pub fn attach(&mut self, observer: Arc<Mutex<dyn Observe<S>>>, frequency: Frequency) {
        self.inner.push((observer, frequency));
    }

    /// Dispatch an event to all eligible observers.
    ///
    /// Observers are filtered using their [`Frequency`] policy.
    pub fn dispatch(
        &self,
        ident: &'static str,
        state: StateView<'_, S>,
        event: &EngineSignal<S::Float>,
    ) {
        let iter = state.iteration();

        for (obs, freq) in &self.inner {
            if !freq.should_run(event, iter) {
                continue;
            }

            let obs = obs.lock().unwrap();
            obs.observe(ident, state, event);
        }
    }
}
