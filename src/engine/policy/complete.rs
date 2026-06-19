//! Completion detection policy.
//!
//! This policy terminates the engine when a computation explicitly reports
//! completion via a [`Progress::Complete`] event in the current event batch.
//!
//! # Behaviour
//!
//! - If any event in the current [`EventBatch`] is
//!   [`Progress::Complete`], returns
//!   [`EngineAction::Stop(Termination::Converged)`].
//! - Otherwise returns [`EngineAction::Continue`].
//!
//! # Design notes
//!
//! This policy is semantic rather than numeric: it relies on the user or
//! solver explicitly signalling completion rather than inferring convergence
//! from tolerances or thresholds.
//!
//! It is useful for:
//! - custom convergence criteria
//! - discrete optimisation
//! - externally-driven solvers
//!
//! It should typically be used alongside numeric termination policies such as
//! [`AbsoluteTolerancePolicy`] or [`TargetValuePolicy`].
//!
use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    progress::Progress,
};

pub struct CompletionPolicy;

impl<F> EnginePolicy<F> for CompletionPolicy {
    fn decide(&mut self, batch: &EventBatch<F>, _context: &EngineContext) -> EngineAction {
        for each in &batch.events {
            if let Progress::Complete = each {
                return EngineAction::Stop(crate::Termination::Converged);
            }
        }

        EngineAction::Continue
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::engine::policy::PolicyStack;
    use crate::progress::Progress;

    #[test]
    fn completion_policy_terminates_when_complete() {
        let mut stack = PolicyStack::<f64>::new().add(CompletionPolicy);

        let batch: EventBatch<f64> = EventBatch::default().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 10,
            ..Default::default()
        };

        assert!(matches!(
            stack.decide(&batch, &ctx),
            EngineAction::Stop(crate::Termination::Converged)
        ))
    }

    #[test]
    fn completion_policy_does_not_terminate_when_not_complete() {
        let mut stack = PolicyStack::<f64>::new().add(CompletionPolicy);

        let batch: EventBatch<f64> = EventBatch::default().add(Progress::ErrorEstimate {
            absolute: 0.0,
            relative: 0.0,
        });
        let ctx = EngineContext {
            iter: 11,
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Continue))
    }
}
