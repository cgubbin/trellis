//! Scheduled checkpoint emission policy.
//!
//! This policy requests periodic checkpoint creation based on the current
//! iteration counter in the [`EngineContext`].
//!
//! It does not terminate execution.
//!
//! # Behaviour
//!
//! - Every `N` iterations (excluding iteration 0), emits
//!   [`EngineAction::EmitCheckpoint(CheckpointReason::Scheduled)`].
//! - Otherwise returns [`EngineAction::Continue`].
//!
//! # Design notes
//!
//! This policy is purely temporal and does not inspect convergence or
//! progress information.
//!
//! It is typically used alongside termination policies such as
//! [`MaxIterationPolicy`] or [`TimeoutPolicy`].
//!
use super::EnginePolicy;

use crate::engine::{event::CheckpointReason, EngineAction, EngineContext, EventBatch};

pub struct CheckpointPolicy {
    every: usize,
}

impl CheckpointPolicy {
    pub fn every(every: usize) -> Self {
        Self { every }
    }
}

impl<F> EnginePolicy<F> for CheckpointPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.iter.is_multiple_of(self.every) & (context.iter > 0) {
            return EngineAction::EmitCheckpoint(CheckpointReason::Scheduled);
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
    fn checkpoint_policy_requests_checkpoint_on_schedule() {
        let mut stack = PolicyStack::<f64>::new().add(CheckpointPolicy::every(10));

        let batch: EventBatch<f64> = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 10,
            ..Default::default()
        };

        assert!(matches!(
            stack.decide(&batch, &ctx),
            EngineAction::EmitCheckpoint(CheckpointReason::Scheduled)
        ))
    }

    #[test]
    fn checkpoint_policy_does_not_request_checkpoint_when_not_on_schedule() {
        let mut stack = PolicyStack::<f64>::new().add(CheckpointPolicy::every(10));

        let batch: EventBatch<f64> = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 11,
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Continue))
    }
}
