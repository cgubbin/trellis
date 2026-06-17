use super::EngineAction;
use crate::state::Progress;
use crate::{State, UserState};

pub trait EnginePolicy<S: UserState> {
    fn next(
        &mut self,
        state: &State<S>,
        progress: Option<Progress<S::Float>>,
        cancelled: bool,
        first_iteration: bool,
    ) -> EngineAction;
}

pub struct DefaultEnginePolicy {
    pub max_iter: usize,
}

impl<S> EnginePolicy<S> for DefaultEnginePolicy
where
    S: UserState,
    S::Float: crate::TrellisFloat,
{
    fn next(
        &mut self,
        state: &State<S>,
        progress: Option<Progress<S::Float>>,
        cancelled: bool,
        first_iteration: bool,
    ) -> EngineAction {
        if first_iteration {
            return EngineAction::Initialise;
        }

        if cancelled {
            return EngineAction::Stop(crate::Termination::Cancelled);
        }

        if state.runtime.iteration() > self.max_iter {
            return EngineAction::Stop(crate::Termination::ExceededMaxIterations);
        }

        if let Some(progress) = progress {
            if let Progress::ErrorEstimate { absolute, .. } = progress {
                if absolute < state.convergence.absolute_tolerance() {
                    return EngineAction::Stop(crate::Termination::Converged);
                }
            }
        }

        EngineAction::Step
    }
}
