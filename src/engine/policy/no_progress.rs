//! # No-progress (stagnation by tolerance) policy
//!
//! Detects lack of meaningful improvement over time.
//!
//! This policy tracks the last observed value and counts how many consecutive
//! iterations fail to improve beyond a given tolerance.
//!
//! ## Behaviour
//!
//! - Extracts numeric values from `Progress::Metric` or
//!   `Progress::ErrorEstimate`.
//! - Computes absolute improvement:
//!   `|current - previous|`
//! - If improvement < `tolerance`, increments stagnation counter.
//! - Otherwise resets counter.
//!
//! ## Termination
//!
//! Stops with [`Termination::Stagnated`] if:
//!
//! - stagnation counter >= `patience`
//!
//! ## Notes
//!
//! This is a *local window heuristic*, not a global stagnation detector.
//! It is sensitive to noise and should be paired with smoothing policies
//! for noisy objectives.
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
    fn decide(&mut self, batch: &EventBatch<F>, _ctx: &EngineContext) -> EngineAction {
        let mut best_in_batch: Option<F> = None;

        for e in &batch.events {
            let value = match e {
                Progress::Metric { value } => *value,
                Progress::ErrorEstimate { absolute, .. } => *absolute,
                _ => continue,
            };

            best_in_batch = Some(match best_in_batch {
                Some(v) => v.min(value),
                None => value,
            });
        }

        if let Some(current) = best_in_batch {
            if let Some(prev) = self.last_value {
                let improvement = (prev - current).abs();

                if improvement < self.tolerance {
                    self.counter += 1;
                } else {
                    self.counter = 0;
                }
            }

            self.last_value = Some(current);
        }

        if self.counter >= self.patience {
            return EngineAction::Stop(Termination::Stagnated);
        }

        EngineAction::Continue
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::policy::PolicyStack;
    use crate::engine::{EngineAction, EngineContext};
    use crate::progress::Progress;

    fn batch(v: f64) -> EventBatch<f64> {
        EventBatch::default().add(Progress::Metric { value: v })
    }

    #[test]
    fn no_progress_resets_with_improvement() {
        let mut stack = PolicyStack::<f64>::new().add(NoProgressPolicy::new(0.1, 3));

        // first: establish baseline
        let batch1 = EventBatch::default().add(Progress::Metric { value: 10.0 });

        let ctx = EngineContext::default();
        let _ = stack.decide(&batch1, &ctx);

        // repeated poor improvement
        for _ in 0..2 {
            let batch = EventBatch::default().add(Progress::Metric { value: 9.95 }); // small improvement

            let res = stack.decide(&batch, &ctx);
            assert!(matches!(res, crate::engine::EngineAction::Continue));
        }

        // real improvement resets
        let reset_batch = EventBatch::default().add(Progress::Metric { value: 8.0 });

        let res = stack.decide(&reset_batch, &ctx);
        assert!(matches!(res, crate::engine::EngineAction::Continue));
    }

    #[test]
    fn no_progress_triggers_stagnation() {
        let mut p = NoProgressPolicy::new(0.1, 2);

        let ctx = EngineContext::default();

        let _ = p.decide(&batch(10.0), &ctx);
        let _ = p.decide(&batch(10.01), &ctx);
        let _ = p.decide(&batch(10.02), &ctx);

        assert!(matches!(
            p.decide(&batch(10.02), &ctx),
            EngineAction::Stop(_)
        ));
    }

    #[test]
    fn ignores_non_numeric_events() {
        let mut p = NoProgressPolicy::new(0.1, 2);

        let mut b = EventBatch::default();
        b.events.push(Progress::Complete);

        let ctx = EngineContext::default();

        assert!(matches!(p.decide(&b, &ctx), EngineAction::Continue));
    }
}
