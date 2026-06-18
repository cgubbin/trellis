use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
};

use num_traits::float::FloatCore;

pub struct AbsoluteTolerancePolicy<F> {
    tolerance: F,
}

impl<F> AbsoluteTolerancePolicy<F> {
    pub fn new(tolerance: F) -> Self {
        Self { tolerance }
    }
}

pub struct RelativeTolerancePolicy<F> {
    tolerance: F,
}

impl<F> RelativeTolerancePolicy<F> {
    pub fn new(tolerance: F) -> Self {
        Self { tolerance }
    }
}

impl<F> EnginePolicy<F> for AbsoluteTolerancePolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            match each {
                Progress::ErrorEstimate { absolute, .. } if *absolute < self.tolerance => {
                    return EngineAction::Stop(crate::Termination::Converged);
                }
                _ => {}
            }
        }

        EngineAction::Continue
    }
}

impl<F> EnginePolicy<F> for RelativeTolerancePolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            match each {
                Progress::ErrorEstimate { relative, .. } if *relative < self.tolerance => {
                    return EngineAction::Stop(crate::Termination::Converged);
                }
                _ => {}
            }
        }

        EngineAction::Continue
    }
}
