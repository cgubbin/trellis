use super::{EnginePolicy, PolicyDecision};

use crate::{
    progress::ProgressReport,
    state::{State, UserState},
    Termination,
};

pub struct CancellationPolicy;

impl<S> EnginePolicy<S> for CancellationPolicy
where
    S: UserState,
{
    fn next(
        &mut self,
        _state: &State<S>,
        _progress: ProgressReport<S::Float>,
        cancelled: bool,
    ) -> PolicyDecision {
        if cancelled {
            return PolicyDecision::Stop(Termination::Cancelled);
        }

        PolicyDecision::Pass
    }
}
