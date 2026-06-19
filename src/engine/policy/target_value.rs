//! # Target value policy
//!
//! Terminates when a metric crosses a predefined target threshold.
//!
//! ## Behaviour
//!
//! - Monitors `Progress::Metric` values.
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::policy::PolicyStack;
    use crate::engine::EngineContext;
    use crate::progress::Progress;

    fn batch(v: f64) -> EventBatch<f64> {
        EventBatch::default().add(Progress::Metric { value: v })
    }

    #[test]
    fn target_reached_stops() {
        let mut p = TargetValuePolicy::new(1.0);

        let ctx = EngineContext::default();

        let res = p.decide(&batch(0.5), &ctx);

        assert!(matches!(res, EngineAction::Stop(_)));
    }

    #[test]
    fn target_not_reached_continues() {
        let mut p = TargetValuePolicy::new(1.0);

        let ctx = EngineContext::default();

        let res = p.decide(&batch(2.0), &ctx);

        assert!(matches!(res, EngineAction::Continue));
    }
}
