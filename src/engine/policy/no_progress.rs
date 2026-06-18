use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
    Termination,
};

use num_traits::float::FloatCore;

pub struct NoProgressPolicy<F> {
    tolerance: F,
    patience: usize,

    last_value: Option<F>,
    counter: usize,
}

impl<F> NoProgressPolicy<F> {
    pub fn new(tolerance: F, patience: usize) -> Self {
        Self {
            tolerance,
            patience,
            last_value: None,
            counter: 0,
        }
    }
}

impl<F> EnginePolicy<F> for NoProgressPolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            let value = match each {
                Progress::Metric { value } => value,
                Progress::ErrorEstimate { absolute, .. } => absolute,
                _ => continue,
            };

            if let Some(previous) = self.last_value {
                let improvement = (previous - *value).abs();

                if improvement < self.tolerance {
                    self.counter += 1;
                } else {
                    self.counter = 0;
                }
            }

            self.last_value = Some(*value);

            if self.counter >= self.patience {
                return EngineAction::Stop(Termination::Stagnated);
            }
        }

        EngineAction::Continue
    }
}
