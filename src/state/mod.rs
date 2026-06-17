mod convergence;
mod policy;
mod progress;
mod runtime;
mod status;

use crate::TrellisFloat;
use convergence::ConvergenceState;
pub(crate) use progress::Progress;
use runtime::RuntimeState;

use num_traits::float::FloatCore;
use web_time::Duration;

pub use policy::{ConvergencePolicy, DefaultConvergencePolicy, StepDecision};
pub use status::{Status, Termination};

#[derive(Clone, Debug)]
pub enum UpdateData<T> {
    // The update can return an estimate of the error
    ErrorEstimate { relative: T, absolute: T },
    // Some calculations do not track an error estimate, this means they converge through a
    // different metric. In this case the user needs to tell trellis convergence has been achieved
    Complete,
}

/// A simple wrapper for error estimates that can be converted to UpdateData
#[derive(Clone, Debug)]
pub struct ErrorEstimate<T>(pub T);

impl<T: Clone> From<ErrorEstimate<T>> for Option<UpdateData<T>> {
    fn from(estimate: ErrorEstimate<T>) -> Self {
        Some(UpdateData::ErrorEstimate {
            relative: estimate.0.clone(),
            absolute: estimate.0,
        })
    }
}

/// The user-defined state must implement this trait to be used as part of the trellis calculation
/// loop
///
/// All other state methods are auto-implemented on a type wrapping the user-defined state.
pub trait UserState: Clone + Default {
    type Float: TrellisFloat;
    type Param;

    // Returns true when the state object is initialised correctly
    fn is_initialised(&self) -> bool {
        true
    }

    // Returns the current parameter value, if one is assigned
    fn get_param(&self) -> Option<&Self::Param>;

    // Called when this iteration was the best iteration seen so far
    fn last_was_best(&mut self) {}

    /// Pure mutation / advancement of user state
    fn update(&mut self);

    /// Reports progress AFTER update
    fn progress(&self) -> Option<Progress<Self::Float>>;
}

/// Trait for controlling convergence behavior
pub trait ConvergenceControl<F: TrellisFloat> {
    /// Update the state object at the end of an iteration
    ///
    /// The update method can be used to control convergence:
    /// - By returning an [`UpdateData::ErrorEstimate`] the error estimate will be compared to the
    ///  solver's absolute and relative tolerances. Termination will happen automatically when these
    ///  conditions are satisfied.
    /// - By returning [`UpdateData::Complete`] the solver will terminate immediately
    /// - By returning [`None`] the solver will continue until max iterations
    fn update(&mut self) -> impl Into<Option<UpdateData<F>>>;
}

/// Automatically implement ConvergenceControl for any UserState that provides an update method
impl<T> ConvergenceControl<T::Float> for T
where
    T: UserState,
    T: UpdateProvider<T::Float>,
{
    fn update(&mut self) -> impl Into<Option<UpdateData<T::Float>>> {
        self.provide_update()
    }
}

/// Helper trait for states that want to provide convergence updates
pub trait UpdateProvider<F: TrellisFloat> {
    fn provide_update(&mut self) -> impl Into<Option<UpdateData<F>>>;
}

/// The state of the [`trellis`] solver
///
/// This contains generic fields common to all solvers, as well as a user-defined state
/// `S` which contains application specific fields.
#[derive(Clone)]
pub struct State<S: UserState> {
    /// The specific component of the state implements the application specific code
    pub(crate) specific: Option<S>,

    pub(crate) runtime: RuntimeState,

    pub(crate) convergence: ConvergenceState<S::Float>,

    /// The last iteration number where the smallest error estimate was found
    last_best_iter: usize,
    /// The current estimate of the error, that observed in the previous iteration
    ///
    /// Note that all stored error values are absolute, to prevent issues at low result values
    error: S::Float,
    /// The estimate of the error observed in the one before last iteration
    prev_error: S::Float,
    /// The best value of the error observed during the entire calculation
    best_error: S::Float,
    /// The second best value of the error observed during the entire calculation
    prev_best_error: S::Float,
    /// The target relative tolerance
    relative_tolerance: S::Float,
    /// The target relative tolerance
    absolute_tolerance: S::Float,
}

impl<S> State<S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    /// Create a new instance of the iteration state
    pub(crate) fn new() -> Self {
        Self {
            specific: Some(S::default()),
            runtime: RuntimeState::new(),
            convergence: ConvergenceState::new(),
            last_best_iter: 0,
            relative_tolerance: <<S as UserState>::Float as FloatCore>::epsilon(),
            absolute_tolerance: <<S as UserState>::Float as FloatCore>::epsilon(),
            error: <<S as UserState>::Float as FloatCore>::infinity(),
            prev_error: <<S as UserState>::Float as FloatCore>::infinity(),
            best_error: <<S as UserState>::Float as FloatCore>::infinity(),
            prev_best_error: <<S as UserState>::Float as FloatCore>::infinity(),
        }
    }

    // TODO: Runtime state -> to be deleted
    pub fn iteration(&self) -> usize {
        self.runtime.iteration()
    }

    pub fn increment_iteration(&mut self) {
        self.runtime.increment_iteration()
    }

    pub fn max_iterations(&self) -> usize {
        self.runtime.max_iterations()
    }

    pub fn set_max_iterations(&mut self, max_iter: usize) {
        self.runtime.set_max_iterations(max_iter)
    }

    pub fn duration(&self) -> Option<&Duration> {
        self.runtime.duration()
    }

    pub fn record_duration(&mut self, duration: Duration) {
        self.runtime.record_duration(duration)
    }

    pub fn termination(&self) -> Option<Termination> {
        self.runtime.termination()
    }

    pub fn terminate(mut self, termination: Termination) -> Self {
        self.runtime.terminate(termination);
        self
    }

    pub fn is_terminated(&self) -> bool {
        self.runtime.is_terminated()
    }

    /// Returns the number of iterations since the best result was observed
    pub(crate) fn iterations_since_best(&self) -> usize {
        self.convergence.iterations_since_best(self.iteration())
    }

    /// Returns the current measure of progress
    pub(crate) fn current(&self) -> S::Float {
        self.convergence.current()
    }

    /// Returns the previous measure of progress
    pub(crate) fn previous(&self) -> S::Float {
        self.convergence.previous()
    }

    /// Returns the best measure of progress
    pub(crate) fn best(&self) -> S::Float {
        self.convergence.best()
    }

    /// Returns the previous best measure of progress
    pub(crate) fn previous_best(&self) -> S::Float {
        self.convergence.previous_best()
    }

    #[must_use]
    /// Set the relative tolerance target
    pub fn set_relative_tolerance(mut self, relative_tolerance: S::Float) -> Self {
        self.convergence.set_relative_tolerance(relative_tolerance);
        self
    }

    #[must_use]
    /// Set the relative tolerance target
    pub fn set_absolute_tolerance(mut self, absolute_tolerance: S::Float) -> Self {
        self.convergence.set_absolute_tolerance(absolute_tolerance);
        self
    }

    pub fn record(&mut self, value: S::Float) -> bool {
        self.convergence.record(value, self.iteration())
    }

    pub fn record_best_if_improved(&mut self) {
        todo!()
    }

    /// Returns true if the state has been initialised. This means a problem specific inner solver
    /// has been attached
    pub(crate) fn is_initialised(&self) -> bool {
        self.specific
            .as_ref()
            .is_some_and(|state| state.is_initialised())
    }

    #[must_use]
    /// Update the state, and the interan state
    pub(crate) fn update(mut self) -> Self {
        // let mut specific = self.specific.take().unwrap();
        // specific.update();
        // match specific.progress() {
        //     // If an error estimate was provided update the internal state accordingly
        //     Some(UpdateData::ErrorEstimate { absolute, .. }) => {
        //         self.error = absolute;
        //         if self.error < self.best_error
        //             || (FloatCore::is_infinite(self.error)
        //                 && FloatCore::is_infinite(self.best_error)
        //                 && FloatCore::is_sign_positive(self.error)
        //                     == FloatCore::is_sign_positive(self.best_error))
        //         {
        //             std::mem::swap(&mut self.prev_best_error, &mut self.best_error);
        //             self.best_error = self.error;
        //             self.last_best_iter = self.iteration();

        //             specific.last_was_best();
        //         }
        //     }
        //     // If the calculation completed successfully return
        //     Some(UpdateData::Complete) => {
        //         return self
        //             .set_specific(specific)
        //             .terminate(Termination::Converged);
        //     }
        //     _ => (),
        // };

        // self = self.set_specific(specific);

        // if self.error < self.absolute_tolerance {
        //     return self.terminate(Termination::Converged);
        // }
        // if self.runtime.exceeded_max_iterations() {
        //     return self.terminate(Termination::ExceededMaxIterations);
        // }

        // self
        let mut specific = self.specific.take().unwrap();

        specific.update();

        self = self.set_specific(specific);

        // ONLY runtime bookkeeping remains here (or soon RuntimeState)
        self
    }

    /// Returns the parameter vector from the inner state variable
    pub(crate) fn get_param(&self) -> Option<&S::Param> {
        self.specific
            .as_ref()
            .and_then(|specific| specific.get_param())
    }

    /// Removes the specific state from the state and returns it
    pub fn take_specific(&mut self) -> S {
        self.specific.take().unwrap()
    }

    #[must_use]
    /// Set the internal state object
    pub fn set_specific(mut self, specific: S) -> Self {
        self.specific = Some(specific);
        self
    }
}
