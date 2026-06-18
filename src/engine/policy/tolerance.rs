use super::{EnginePolicy, PolicyDecision};

use crate::{
    progress::{Progress, ProgressReport},
    state::{State, UserState},
    Termination,
};

pub struct AbsoluteTolerancePolicy<F> {
    tolerance: F,
}

impl<F> AbsoluteTolerancePolicy<F> {
    pub fn new(tolerance: F) -> Self {
        Self { tolerance }
    }
}

pub struct RelativeTolerancePolicy<F> {
    tolerance: F,
}

impl<F> RelativeTolerancePolicy<F> {
    pub fn new(tolerance: F) -> Self {
        Self { tolerance }
    }
}

impl<S> EnginePolicy<S> for AbsoluteTolerancePolicy<S::Float>
where
    S: UserState,
    S::Float: crate::TrellisFloat,
{
    fn next(
        &mut self,
        state: &State<S>,
        progress: ProgressReport<S::Float>,
        cancelled: bool,
    ) -> PolicyDecision {
        if let Progress::ErrorEstimate { absolute, .. } = progress.measure {
            if absolute < self.tolerance {
                return PolicyDecision::Stop(crate::Termination::Converged);
            }
        }

        PolicyDecision::Pass
    }
}

impl<S> EnginePolicy<S> for RelativeTolerancePolicy<S::Float>
where
    S: UserState,
    S::Float: crate::TrellisFloat,
{
    fn next(
        &mut self,
        state: &State<S>,
        progress: ProgressReport<S::Float>,
        cancelled: bool,
    ) -> PolicyDecision {
        if let Progress::ErrorEstimate { relative, .. } = progress.measure {
            if relative < self.tolerance {
                return PolicyDecision::Stop(crate::Termination::Converged);
            }
        }

        PolicyDecision::Pass
    }
}
