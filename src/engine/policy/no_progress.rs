//! # No-progress (stagnation by tolerance) policy
//!
//! Detects lack of meaningful improvement over time.
//!
//! This policy tracks the last observed value and counts how many consecutive
//! iterations fail to improve beyond a given tolerance.
//!
//! ## Behaviour
//!
//! - Extracts numeric values from `Progress::Measure`
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

    best_so_far: Option<F>,
    counter: usize,
}

impl<F> NoProgressPolicy<F> {
    pub fn new(tolerance: F, patience: usize) -> Self {
        Self {
            tolerance,
            patience,
            best_so_far: None,
            counter: 0,
        }
    }
}

impl<F> EnginePolicy<F> for NoProgressPolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _ctx: &EngineContext) -> EngineAction {
        let mut batch_best: Option<F> = None;

        for e in &batch.events {
            let value = match e {
                Progress::Measure(v) => *v,
                Progress::Report { measure, .. } => *measure,
                _ => continue,
            };

            batch_best = Some(match batch_best {
                Some(v) => v.min(value),
                None => value,
            });
        }

        let Some(batch_best) = batch_best else {
            return EngineAction::Continue;
        };

        match self.best_so_far {
            None => {
                self.best_so_far = Some(batch_best);
                self.counter = 0;
                return EngineAction::Continue;
            }

            Some(prev_best) => {
                let denom = prev_best.abs().max(F::one());
                let improvement = (prev_best - batch_best) / denom;

                if improvement > self.tolerance {
                    // meaningful improvement → reset patience
                    self.best_so_far = Some(batch_best);
                    self.counter = 0;
                } else {
                    // no meaningful improvement
                    self.counter += 1;
                }
            }
        }

        if self.counter >= self.patience {
            return EngineAction::Stop(crate::Termination::NoProgress);
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
        EventBatch::new().add(Progress::Measure(v))
    }

    #[test]
    fn no_progress_resets_with_improvement() {
        let mut stack = PolicyStack::<f64>::new().add(NoProgressPolicy::new(0.1, 3));

        // first: establish baseline
        let batch1 = EventBatch::new().add(Progress::Measure(10.0));

        let ctx = EngineContext::default();
        let _ = stack.decide(&batch1, &ctx);

        // repeated poor improvement
        for _ in 0..2 {
            let batch = EventBatch::new().add(Progress::Measure(9.95)); // small improvement

            let res = stack.decide(&batch, &ctx);
            assert!(matches!(res, crate::engine::EngineAction::Continue));
        }

        // real improvement resets
        let reset_batch = EventBatch::new().add(Progress::Measure(8.0));

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

        let mut b = EventBatch::new();
        b.events.push(Progress::Complete);

        let ctx = EngineContext::default();

        assert!(matches!(p.decide(&b, &ctx), EngineAction::Continue));
    }
}
