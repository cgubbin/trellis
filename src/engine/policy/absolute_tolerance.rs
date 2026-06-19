//! # Absolute tolerance policy
//!
//! Terminates when the absolute error estimate falls below a threshold.
//!
//! ## Behaviour
//!
//! - Consumes `Progress::ErrorEstimate` events.
//! - Checks `absolute < tolerance`.
//!
//! ## Termination
//!
//! Returns [`Termination::Converged`] when condition is met.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::policy::PolicyStack;
    use crate::engine::{EngineContext, EventBatch};
    use crate::progress::Progress;

    #[test]
    fn absolute_tolerance_stops_on_error_below_threshold() {
        let mut stack = PolicyStack::<f64>::new().add(AbsoluteTolerancePolicy::new(0.1));

        let batch = EventBatch::default().add(Progress::ErrorEstimate {
            absolute: 0.05,
            relative: 0.2,
        });

        let ctx = EngineContext::default();

        assert!(matches!(
            stack.decide(&batch, &ctx),
            crate::engine::EngineAction::Stop(_)
        ));
    }

    #[test]
    fn absolute_tolerance_continues_above_threshold() {
        let mut stack = PolicyStack::<f64>::new().add(AbsoluteTolerancePolicy::new(0.1));

        let batch = EventBatch::default().add(Progress::ErrorEstimate {
            absolute: 0.5,
            relative: 0.2,
        });

        let ctx = EngineContext::default();

        assert!(matches!(
            stack.decide(&batch, &ctx),
            crate::engine::EngineAction::Continue
        ));
    }
}
