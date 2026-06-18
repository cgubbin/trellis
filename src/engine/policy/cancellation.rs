use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
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
        _events: &[RawEvent<S::Float>],
        cancelled: bool,
    ) -> EngineEvent<S::Float> {
        if cancelled {
            return EngineEvent::TerminationRequested(Termination::Cancelled);
        }

        EngineEvent::Pass
    }
}
