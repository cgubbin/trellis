//! # Target value policy
//!
//! Terminates when a metric crosses a predefined target threshold.
//!
//! ## Behaviour
//!
//! - Monitors `Progress::Measure` values.
//! - If any value <= `target`, termination is triggered.
//!
//! ## Termination
//!
//! Returns [`Termination::Converged`] when target is reached.
//!
//! ## Notes
//!
//! This policy assumes minimisation semantics.
//! For maximisation problems, invert the metric externally.
use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
    Termination,
};

use num_traits::float::FloatCore;

pub struct TargetValuePolicy<F> {
    target: F,
    tolerance: F,
    window: Vec<F>,
    window_size: usize,
}

impl<F: FloatCore> TargetValuePolicy<F> {
    pub fn new(target: F, tolerance: F, window_size: usize) -> Self {
        Self {
            target,
            tolerance,
            window: Vec::with_capacity(window_size),
            window_size,
        }
    }
}

impl<F> EnginePolicy<F> for TargetValuePolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for event in &batch.events {
            let value = match event {
                Progress::Measure(v) => *v,
                Progress::Report { measure, .. } => *measure,
                _ => continue,
            };

            // symmetric distance to target
            let dist = (value - self.target).abs();

            self.window.push(dist);

            if self.window.len() > self.window_size {
                self.window.remove(0);
            }
        }

        if self.window.len() < self.window_size {
            return EngineAction::Continue;
        }

        // mean absolute distance
        let mean = self.window.iter().copied().fold(F::zero(), |a, b| a + b)
            / F::from(self.window.len()).unwrap();

        // tolerance-based stopping condition
        if mean < self.tolerance {
            return EngineAction::Stop(Termination::Converged);
        }

        EngineAction::Continue
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::EngineContext;
    use crate::progress::Progress;

    fn batch(v: f64) -> EventBatch<f64> {
        EventBatch::new().add(Progress::Measure(v))
    }

    #[test]
    fn target_reached_stops() {
        let mut p = TargetValuePolicy::new(1.0, 0.01, 5);

        let ctx = EngineContext::default();

        for _ in 0..5 {
            p.decide(&batch(1.0), &ctx);
        }

        let res = p.decide(&batch(1.0), &ctx);

        assert!(matches!(res, EngineAction::Stop(_)));
    }

    #[test]
    fn target_not_reached_continues() {
        let mut p = TargetValuePolicy::new(1.0, 0.01, 5);

        let ctx = EngineContext::default();

        let res = p.decide(&batch(2.0), &ctx);

        assert!(matches!(res, EngineAction::Continue));
    }
}
