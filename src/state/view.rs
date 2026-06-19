use super::{ConvergenceState, RuntimeState, State, UserState};
use crate::engine::Termination;

use num_traits::float::FloatCore;
use std::time::Duration;

#[derive(Copy, Clone)]
pub struct StateView<'a, S: UserState> {
    state: &'a State<S>,
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
    pub fn iteration(&self) -> usize {
        self.state.runtime.iteration()
    }

    pub fn duration(&self) -> Option<Duration> {
        self.state.runtime.duration().copied()
    }

    pub fn termination(&self) -> Option<Termination> {
        self.state.runtime.termination()
    }

    pub fn best_measure(&self) -> S::Float {
        self.state.convergence.best()
    }

    pub fn current_measure(&self) -> S::Float {
        self.state.convergence.current()
    }

    pub fn iterations_since_best(&self) -> usize {
        self.state
            .convergence
            .iterations_since_best(self.state.runtime.iteration())
    }

    pub fn user(&self) -> &S {
        &self.state.user
    }
}
