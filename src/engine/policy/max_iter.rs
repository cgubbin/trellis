use super::EnginePolicy;

use crate::engine::{EngineAction, EngineContext, EventBatch};

pub struct MaxIterationPolicy {
    max_iters: usize,
}

impl MaxIterationPolicy {
    pub fn new(max_iters: usize) -> Self {
        Self { max_iters }
    }
}

impl<F> EnginePolicy<F> for MaxIterationPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.iter > self.max_iters {
            return EngineAction::Stop(crate::Termination::ExceededMaxIterations);
        }

        EngineAction::Continue
    }
}
