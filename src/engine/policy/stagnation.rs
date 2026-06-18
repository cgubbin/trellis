use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::ProgressReport,
    state::{State, UserState},
    Termination,
};

pub struct StagnationPolicy {
    patience: usize,
}

impl StagnationPolicy {
    pub fn new(patience: usize) -> Self {
        Self { patience }
    }
}

impl<S> EnginePolicy<S> for StagnationPolicy
where
    S: UserState,
{
    fn next(
        &mut self,
        state: &State<S>,
        _events: &[RawEvent<S::Float>],
        _cancelled: bool,
    ) -> EngineEvent<S::Float> {
        if state.iterations_since_best() >= self.patience {
            return EngineEvent::TerminationRequested(Termination::Stagnated);
        }

        EngineEvent::Pass
    }
}
