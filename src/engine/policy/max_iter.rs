use super::{EnginePolicy, PolicyDecision};

use crate::{
    progress::ProgressReport,
    state::{State, UserState},
    Termination,
};

pub struct MaxIterationPolicy {
    max_iters: usize,
}

impl MaxIterationPolicy {
    pub fn new(max_iters: usize) -> Self {
        Self { max_iters }
    }
}

impl<S> EnginePolicy<S> for MaxIterationPolicy
where
    S: UserState,
{
    fn next(
        &mut self,
        state: &State<S>,
        _progress: ProgressReport<S::Float>,
        cancelled: bool,
    ) -> PolicyDecision {
        if state.runtime.iteration() > self.max_iters {
            return PolicyDecision::Stop(crate::Termination::ExceededMaxIterations);
        }
        PolicyDecision::Pass
    }
}
