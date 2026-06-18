use std::time::Duration;

use super::{EnginePolicy, PolicyDecision};

use crate::{
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
        _progress: ProgressReport<S::Float>,
        _cancelled: bool,
    ) -> PolicyDecision {
        if let Some(elapsed) = state.runtime.duration() {
            if *elapsed >= self.timeout {
                return PolicyDecision::Stop(Termination::Timeout);
            }
        }

        PolicyDecision::Pass
    }
}
