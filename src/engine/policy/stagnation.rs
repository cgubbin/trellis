use super::{EnginePolicy, PolicyDecision};

use crate::{
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
        _progress: ProgressReport<S::Float>,
        _cancelled: bool,
    ) -> PolicyDecision {
        if state.iterations_since_best() >= self.patience {
            return PolicyDecision::Stop(Termination::Stagnated);
        }

        PolicyDecision::Pass
    }
}
