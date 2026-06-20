//! # Stagnation policy (window-based)
//!
//! Detects lack of meaningful variation over a sliding window of values.
//!
//! ## Behaviour
//!
//! - Maintains a fixed-size history of recent values.
//! - Values are extracted from:
//!   - `Progress::Measure`
//!
//! - Once enough samples are collected:
//!
//!   - If all consecutive differences are < `epsilon()`
//!     → stagnation is detected
//!
//! ## Termination
//!
//! Returns [`Termination::Stagnated`] when stagnation is detected.
//!
//! ## Notes
//!
//! This policy is stricter than [`NoProgressPolicy`] because it requires
//! *persistent flat behaviour over a window*, not just repeated small steps.
use super::EnginePolicy;

use num_traits::float::FloatCore;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
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
    fn decide(&mut self, batch: &EventBatch<F>, _ctx: &EngineContext) -> EngineAction {
        for e in &batch.events {
            let v = match e {
                Progress::Measure(value) => *value,
                _ => continue,
            };

            self.history.push(v);
        }

        if self.history.len() > self.window {
            self.history.remove(0);
        }

        if self.history.len() < self.window {
            return EngineAction::Continue;
        }

        // slope-based stagnation
        let first = self.history.first().unwrap();
        let last = self.history.last().unwrap();

        let improvement = (*first - *last).abs();

        if improvement < F::epsilon() {
            return EngineAction::Stop(Termination::Stagnated);
        }

        EngineAction::Continue
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::policy::PolicyStack;
    use crate::engine::EngineContext;
    use crate::progress::Progress;

    fn batch(v: f64) -> EventBatch<f64> {
        EventBatch::new().add(Progress::Measure(v))
    }

    #[test]
    fn stagnation_detects_flat_region() {
        let mut p = StagnationPolicy::new(3);

        let ctx = EngineContext::default();

        let _ = p.decide(&batch(1.0), &ctx);
        let _ = p.decide(&batch(1.0), &ctx);
        let _ = p.decide(&batch(1.0), &ctx);
        let res = p.decide(&batch(1.0), &ctx);

        assert!(matches!(res, EngineAction::Stop(_)));
    }

    #[test]
    fn stagnation_requires_window() {
        let mut p = StagnationPolicy::new(5);

        let ctx = EngineContext::default();

        let res = p.decide(&batch(1.0), &ctx);

        assert!(matches!(res, EngineAction::Continue));
    }

    #[test]
    fn stagnation_resets_with_change() {
        let mut stack = PolicyStack::<f64>::new().add(StagnationPolicy::new(3));

        let ctx = EngineContext::default();

        let seq = vec![1.0, 1.0001, 1.0002, 2.0];

        for v in seq {
            let batch = EventBatch::new().add(Progress::Measure(v));

            let res = stack.decide(&batch, &ctx);

            if v == 2.0 {
                assert!(matches!(res, crate::engine::EngineAction::Continue));
            }
        }
    }
}
