use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::{Progress, ProgressReport},
    state::{State, UserState},
};

pub struct CompletionPolicy;

impl<S> EnginePolicy<S> for CompletionPolicy
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
            if RawEvent::Progress(Progress::Complete) == *each {
                return EngineEvent::TerminationRequested(crate::Termination::Converged);
            }
        }

        EngineEvent::Pass
    }
}
