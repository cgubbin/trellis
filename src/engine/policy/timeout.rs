use std::time::Duration;

use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::ProgressReport,
    state::{State, UserState},
    Termination,
};

pub struct TimeoutPolicy {
    timeout: Duration,
}

impl TimeoutPolicy {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl<S> EnginePolicy<S> for TimeoutPolicy
where
    S: UserState,
{
    fn next(
        &mut self,
        state: &State<S>,
        _events: &[RawEvent<S::Float>],
        _cancelled: bool,
    ) -> EngineEvent<S::Float> {
        if let Some(elapsed) = state.runtime.duration() {
            if *elapsed >= self.timeout {
                return EngineEvent::TerminationRequested(Termination::Timeout);
            }
        }

        EngineEvent::Pass
    }
}
