use super::{EnginePolicy, PolicyDecision};

use crate::{
    progress::{Progress, ProgressReport},
    state::{State, UserState},
    Termination,
};

pub struct CompletionPolicy;

impl<S> EnginePolicy<S> for CompletionPolicy
where
    S: UserState,
{
    fn next(
        &mut self,
        _state: &State<S>,
        progress: ProgressReport<S::Float>,
        _cancelled: bool,
    ) -> PolicyDecision {
        if Progress::Complete == progress.measure {
            return PolicyDecision::Stop(crate::Termination::Converged);
        }

        PolicyDecision::Pass
    }
}
