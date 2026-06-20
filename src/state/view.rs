use super::{ConvergenceState, RuntimeState, State, UserState};

use num_traits::float::FloatCore;
use std::time::Duration;

/// Read-only view into engine state at a single point in time.
///
/// This type is used by:
/// - observers
/// - logging systems
/// - extensions
/// - external monitoring hooks
///
/// It provides **safe immutable access** without exposing ownership
/// of the underlying `State<S>`.
pub struct StateView<'a, S: UserState> {
    state: &'a State<S>,
}

impl<'a, S: UserState> Copy for StateView<'a, S> {}

impl<'a, S: UserState> Clone for StateView<'a, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, S: UserState> StateView<'a, S> {
    pub(crate) fn new(state: &'a State<S>) -> Self {
        Self { state }
    }
}

impl<'a, S> StateView<'a, S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    /// Current iteration number of the engine.
    pub fn iteration(&self) -> usize {
        self.state.runtime.iteration()
    }

    /// Total elapsed execution duration.
    pub fn duration(&self) -> Duration {
        self.state.runtime.duration()
    }

    /// Best convergence value observed so far.
    pub fn best_measure(&self) -> S::Float {
        self.state.convergence.best()
    }

    /// Current convergence value.
    pub fn current_measure(&self) -> S::Float {
        self.state.convergence.current()
    }

    /// Number of iterations since the last improvement.
    pub fn iterations_since_best(&self) -> usize {
        self.state
            .convergence
            .iterations_since_best(self.state.runtime.iteration())
    }

    /// Access to user-defined state (read-only).
    pub(crate) fn user<'b>(&'b self) -> &'a S {
        &self.state.user
    }

    /// Access to runtime state (read-only).
    pub(crate) fn runtime<'b>(&'b self) -> &'a RuntimeState {
        &self.state.runtime
    }

    /// Access to convergence state (read-only).
    pub(crate) fn convergence<'b>(&'b self) -> &'a ConvergenceState<S::Float> {
        &self.state.convergence
    }
}
