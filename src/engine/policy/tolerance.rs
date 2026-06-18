use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::{Progress, ProgressReport},
    state::{State, UserState},
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
        events: &[RawEvent<S::Float>],
        cancelled: bool,
    ) -> EngineEvent<S::Float> {
        for each in events {
            match each {
                RawEvent::Progress(Progress::ErrorEstimate { absolute, .. })
                    if *absolute < self.tolerance =>
                {
                    return EngineEvent::TerminationRequested(crate::Termination::Converged);
                }
                _ => {}
            }
        }

        EngineEvent::Pass
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
        events: &[RawEvent<S::Float>],
        cancelled: bool,
    ) -> EngineEvent<S::Float> {
        for each in events {
            match each {
                RawEvent::Progress(Progress::ErrorEstimate { relative, .. })
                    if *relative < self.tolerance =>
                {
                    return EngineEvent::TerminationRequested(crate::Termination::Converged);
                }
                _ => {}
            }
        }

        EngineEvent::Pass
    }
}
