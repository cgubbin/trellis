use std::time::Duration;

use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
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

impl<F> EnginePolicy<F> for TimeoutPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.elapsed >= self.timeout {
            return EngineAction::Stop(Termination::Timeout);
        }

        EngineAction::Continue
    }
}
