#![allow(clippy::type_complexity)]

use crate::engine::EngineSignal;
use crate::state::{StateView, UserState};

use std::sync::{Arc, Mutex};

#[cfg(feature = "writing")]
mod csv_file;

mod failure;
mod metrics;

#[cfg(feature = "plotting")]
mod plot;

mod sampler;
mod tracing;

#[cfg(feature = "writing")]
pub use csv_file::CsvProgressWriter;

#[cfg(feature = "plotting")]
pub use plot::PlotObserver;

pub use tracing::Tracer;

/// Core observer trait for the engine event system.
///
/// Observers receive a stream of signal events during execution
/// along with a read-only view over the iteration state
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        CancellationGuard, GenerateBuilder, MaxIterationPolicy, Procedure, Progress,
        ProgressDiagnostics, Snapshotable, StateRestorer, UserState,
    };

    #[derive(Clone, Debug)]
    pub struct DummyProblem {
        pub target: f64,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct DummySnapshot {
        pub value: f64,
        pub steps: usize,
    }

    #[derive(Clone, Debug)]
    pub struct DummyState {
        pub value: f64,
        pub steps: usize,
    }

    impl Default for DummyState {
        fn default() -> Self {
            Self {
                value: 10.0,
                steps: 0,
            }
        }
    }

    impl UserState for DummyState {
        type Float = f64;

        fn progress(&self) -> Progress<Self::Float> {
            Progress::Report {
                measure: self.value,
                diagnostics: ProgressDiagnostics {
                    absolute_error: Some(self.value.abs()),
                    relative_error: Some(self.value.abs() / 10.0),
                    ..Default::default()
                },
            }
        }
    }

    impl Snapshotable for DummyState {
        type Snapshot = DummySnapshot;

        fn snapshot(&self) -> Self::Snapshot {
            DummySnapshot {
                value: self.value,
                steps: self.steps,
            }
        }
    }

    impl StateRestorer<DummyState> for DummyState {
        fn restore(snapshot: DummySnapshot) -> Self {
            Self {
                value: snapshot.value,
                steps: snapshot.steps,
            }
        }
    }

    pub struct DummyProcedure;

    impl Procedure<DummyProblem> for DummyProcedure {
        const NAME: &'static str = "Dummy Procedure";

        type State = DummyState;
        type Output = DummyState;

        fn initialise(&self, _problem: &mut DummyProblem, _state: &mut Self::State) {}

        fn step(
            &self,
            problem: &mut DummyProblem,
            state: &mut Self::State,
            _guard: CancellationGuard<'_>,
        ) {
            state.steps += 1;

            let delta = state.value - problem.target;

            if delta.abs() > 1e-12 {
                state.value -= 0.5 * delta;
            }
        }

        fn finalise(&self, _problem: &mut DummyProblem, state: &Self::State) -> Self::Output {
            state.clone()
        }
    }

    #[derive(Default)]
    struct Spy {
        events: std::sync::Mutex<Vec<&'static str>>,
    }

    impl<S> Observe<S> for Spy
    where
        S: UserState,
    {
        fn observe(
            &self,
            _name: &'static str,
            _state: StateView<'_, S>,
            event: &EngineSignal<S::Float>,
        ) {
            self.events.lock().unwrap().push(event.as_tag());
        }
    }

    #[test]
    fn observer_receives_lifecycle_events() {
        let spy = std::sync::Arc::new(Spy::default());
        let target = 1.0;

        let _ = DummyProcedure
            .build_for(DummyProblem { target })
            .with_initial_state(DummyState::default())
            .attach_observer(spy.clone(), Frequency::Always)
            .and_policy(MaxIterationPolicy::new(3))
            .finalise()
            .run();

        let events = spy.events.lock().unwrap();

        assert!(events.contains(&"initialised"));
        assert!(events.contains(&"progress"));
        assert!(events.contains(&"termination"));
    }

    #[test]
    fn observer_frequency_every_filters_progress_events() {
        let spy = std::sync::Arc::new(Spy::default());

        let target = 1.0;
        let _ = DummyProcedure
            .build_for(DummyProblem { target })
            .with_initial_state(DummyState::default())
            .attach_observer(spy.clone(), Frequency::Every(2))
            .and_policy(MaxIterationPolicy::new(5))
            .finalise()
            .run();

        let events = spy.events.lock().unwrap();

        let progress_count = events.iter().filter(|e| **e == "progress").count();

        assert_eq!(progress_count, 2);
    }
}
