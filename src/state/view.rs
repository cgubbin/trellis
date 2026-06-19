use super::{ConvergenceState, RuntimeState, State, UserState};
use crate::engine::Termination;

use num_traits::float::FloatCore;
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct StateView<'a, S: UserState> {
    convergence: &'a ConvergenceState<S::Float>,
    runtime: &'a RuntimeState,
    snapshot: S::Snapshot,
    phantom: std::marker::PhantomData<S>,
}

impl<'a, S: UserState> StateView<'a, S> {
    pub(crate) fn new(state: &'a State<S>) -> Self {
        Self {
            convergence: &state.convergence,
            runtime: &state.runtime,
            snapshot: state.user.snapshot(),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, S> StateView<'a, S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    pub fn iteration(&self) -> usize {
        self.runtime.iteration()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.runtime.duration().copied()
    }

    pub fn termination(&self) -> Option<Termination> {
        self.runtime.termination()
    }

    pub fn best_measure(&self) -> S::Float {
        self.convergence.best()
    }

    pub fn current_measure(&self) -> S::Float {
        self.convergence.current()
    }

    pub fn iterations_since_best(&self) -> usize {
        self.convergence
            .iterations_since_best(self.runtime.iteration())
    }

    pub fn snapshot(&self) -> &S::Snapshot {
        &self.snapshot
    }
}
