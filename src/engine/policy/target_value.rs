use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
    Termination,
};

use num_traits::float::FloatCore;

pub struct TargetValuePolicy<F> {
    target: F,
}

impl<F> TargetValuePolicy<F> {
    pub fn new(target: F) -> Self {
        Self { target }
    }
}

impl<F> EnginePolicy<F> for TargetValuePolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            match each {
                Progress::Metric { value } if *value <= self.target => {
                    return EngineAction::Stop(Termination::Converged);
                }
                _ => {}
            }
        }
        EngineAction::Continue
    }
}
