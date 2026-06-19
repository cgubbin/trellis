use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
};

pub struct CompletionPolicy;

impl<F> EnginePolicy<F> for CompletionPolicy {
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            if let Progress::Complete = each {
                return EngineAction::Stop(crate::Termination::Converged);
            }
        }

        EngineAction::Continue
    }
}
