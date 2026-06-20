//! # Relative tolerance policy
//!
//! Terminates when the relative error estimate falls below a threshold.
//!
//! ## Behaviour
//!
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

pub struct RelativeTolerancePolicy<F> {
    tolerance: F,
}

impl<F> RelativeTolerancePolicy<F> {
    pub fn new(tolerance: F) -> Self {
        Self { tolerance }
    }
}

impl<F> EnginePolicy<F> for RelativeTolerancePolicy<F>
where
    F: FloatCore,
{
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            if let Progress::Report { diagnostics, .. } = each {
                if let Some(rel) = diagnostics.relative_error {
                    if rel < self.tolerance {
                        return EngineAction::Stop(crate::Termination::Converged);
                    }
                }
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
    use crate::progress::{Progress, ProgressDiagnostics};

    #[test]
    fn relative_tolerance_stops_on_small_relative_error() {
        let mut stack = PolicyStack::<f64>::new().add(RelativeTolerancePolicy::new(0.1));

        let batch = EventBatch::new().add(Progress::Report {
            measure: 1.0,
            diagnostics: ProgressDiagnostics {
                relative_error: Some(0.05),
                ..Default::default()
            },
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

        let batch = EventBatch::new().add(Progress::Report {
            measure: 1.0,
            diagnostics: ProgressDiagnostics {
                relative_error: Some(0.5),
                ..Default::default()
            },
        });

        let ctx = EngineContext::default();

        assert!(matches!(
            stack.decide(&batch, &ctx),
            crate::engine::EngineAction::Continue
        ));
    }
}
