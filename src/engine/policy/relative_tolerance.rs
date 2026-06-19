//! # Relative tolerance policy
//!
//! Terminates when the relative error estimate falls below a threshold.
//!
//! ## Behaviour
//!
//! - Consumes `Progress::ErrorEstimate` events.
//! - Checks `relative < tolerance`.
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
    fn relative_tolerance_stops_on_small_relative_error() {
        let mut stack = PolicyStack::<f64>::new().add(RelativeTolerancePolicy::new(0.1));

        let batch = EventBatch::new().add(Progress::ErrorEstimate {
            absolute: 0.5,
            relative: 0.05,
        });

        let ctx = EngineContext::default();

        assert!(matches!(
            stack.decide(&batch, &ctx),
            crate::engine::EngineAction::Stop(_)
        ));
    }

    #[test]
    fn relative_tolerance_continues_when_large() {
        let mut stack = PolicyStack::<f64>::new().add(RelativeTolerancePolicy::new(0.1));

        let batch = EventBatch::new().add(Progress::ErrorEstimate {
            absolute: 0.5,
            relative: 0.5,
        });

        let ctx = EngineContext::default();

        assert!(matches!(
            stack.decide(&batch, &ctx),
            crate::engine::EngineAction::Continue
        ));
    }
}
