use super::EnginePolicy;

use crate::engine::{EngineAction, EngineContext, EventBatch};

pub struct CheckpointPolicy {
    every: usize,
}

impl CheckpointPolicy {
    pub fn new(every: usize) -> Self {
        Self { every }
    }
}

impl<F> EnginePolicy<F> for CheckpointPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.iter.is_multiple_of(self.every) & (context.iter > 0) {
            return EngineAction::EmitCheckpoint;
        }

        EngineAction::Continue
    }
}
