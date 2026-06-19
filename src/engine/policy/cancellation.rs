//! Cancellation-based termination policy.
//!
//! This policy terminates execution immediately when the engine context
//! indicates that cancellation has been requested.
//!
//! It does not inspect progress events or iteration state.
//!
//! # Behaviour
//!
//! - If `context.cancelled == true`, returns
//!   [`EngineAction::Stop(Termination::Cancelled)`].
//! - Otherwise returns [`EngineAction::Continue`].
//!
//! # Design notes
//!
//! This policy should almost always be included as the first policy in a
//! [`PolicyStack`], ensuring user cancellation has highest priority.
//!
//! It is intentionally stateless.
use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    Termination,
};

pub struct CancellationPolicy;

impl<F> EnginePolicy<F> for CancellationPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.cancelled {
            return EngineAction::Stop(Termination::Cancelled);
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
    fn cancellation_policy_stops_when_cancelled() {
        let mut stack = PolicyStack::<f64>::new().add(CancellationPolicy);

        let batch: EventBatch<f64> = EventBatch::default().add(Progress::Complete);
        let ctx = EngineContext {
            cancelled: true,
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Stop(_)));
    }

    #[test]
    fn cancellation_policy_runs_when_not_cancelled() {
        let mut stack = PolicyStack::<f64>::new().add(CancellationPolicy);

        let batch: EventBatch<f64> = EventBatch::default().add(Progress::Complete);
        let ctx = EngineContext {
            cancelled: false,
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Continue));
    }
}
