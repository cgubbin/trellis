use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
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
        events: &[RawEvent<S::Float>],
        _cancelled: bool,
    ) -> EngineEvent<S::Float> {
        for each in events {
            match each {
                RawEvent::Progress(Progress::Metric { value }) if *value <= self.target => {
                    return EngineEvent::TerminationRequested(Termination::Converged);
                }
                _ => {}
            }
        }
        EngineEvent::Pass
    }
}
