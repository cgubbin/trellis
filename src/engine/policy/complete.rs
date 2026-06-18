use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
};

pub struct CompletionPolicy;

impl<F> EnginePolicy<F> for CompletionPolicy {
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            match each {
                Progress::Complete => {
                    return EngineAction::Stop(crate::Termination::Converged);
                }
                _ => {}
            }
        }

        EngineAction::Continue
    }
}
