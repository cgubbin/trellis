use super::{EnginePolicy, PolicyDecision};

use crate::{
    progress::{Progress, ProgressReport},
    state::{State, UserState},
    Termination,
};

pub struct TargetValuePolicy<F> {
    target: F,
}

impl<F> TargetValuePolicy<F> {
    pub fn new(target: F) -> Self {
        Self { target }
    }
}

impl<S> EnginePolicy<S> for TargetValuePolicy<S::Float>
where
    S: UserState,
{
    fn next(
        &mut self,
        _state: &State<S>,
        progress: ProgressReport<S::Float>,
        _cancelled: bool,
    ) -> PolicyDecision {
        match progress.measure {
            Progress::Metric { value } if value <= self.target => {
                PolicyDecision::Stop(Termination::Converged)
            }

            _ => PolicyDecision::Pass,
        }
    }
}
