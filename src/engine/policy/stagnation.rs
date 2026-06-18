use super::EnginePolicy;

use num_traits::float::FloatCore;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
    state::{State, UserState},
    Termination,
};

pub struct StagnationPolicy<F> {
    window: usize,
    history: Vec<F>,
}

impl<F> StagnationPolicy<F> {
    pub fn new(window: usize) -> Self {
        Self {
            window,
            history: Vec::new(),
        }
    }
}

impl<F: FloatCore> EnginePolicy<F> for StagnationPolicy<F> {
    fn decide(&mut self, batch: &EventBatch<F>, ctx: &EngineContext) -> EngineAction {
        for e in &batch.events {
            match e {
                Progress::Metric { value } => {
                    self.history.push(*value);
                }
                Progress::ErrorEstimate { absolute, .. } => {
                    self.history.push(*absolute);
                }
                _ => {}
            }
        }

        // keep bounded window
        if self.history.len() > self.window {
            self.history.drain(0..1);
        }

        // need enough history before deciding
        if self.history.len() < self.window {
            return EngineAction::Continue;
        }

        // stagnation test (monotonic or flat improvement)
        let stagnating = self
            .history
            .windows(2)
            .all(|w| (w[1] - w[0]).abs() < F::epsilon());

        if stagnating {
            return EngineAction::Stop(Termination::Stagnated);
        }

        EngineAction::Continue
    }
}
