use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    Termination,
};

pub struct CancellationPolicy;

impl<F> EnginePolicy<F> for CancellationPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.cancelled {
            return EngineAction::Stop(Termination::Cancelled);
        }

        EngineAction::Continue
    }
}
