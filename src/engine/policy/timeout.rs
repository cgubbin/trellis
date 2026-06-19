//! # Timeout policy
//!
//! Terminates execution after a wall-clock duration has elapsed.
//!
//! ## Behaviour
//!
//! - Compares `EngineContext.elapsed` against a fixed timeout.
//! - Does not inspect progress or iteration state.
//!
//! ## Termination
//!
//! Returns [`Termination::Timeout`] when elapsed >= timeout.
//!
//! ## Notes
//!
//! This policy is orthogonal to convergence logic and should always be
//! included for safety in long-running computations.
use std::time::Duration;

use super::EnginePolicy;

use crate::{
    engine::{EngineAction, EngineContext, EventBatch},
    Termination,
};

pub struct TimeoutPolicy {
    timeout: Duration,
}

impl TimeoutPolicy {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl<F> EnginePolicy<F> for TimeoutPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.elapsed >= self.timeout {
            return EngineAction::Stop(Termination::Timeout);
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
    fn timeout_policy_terminates_for_durations_greater_than_limit() {
        let mut stack = PolicyStack::<f64>::new().add(TimeoutPolicy::new(Duration::new(5, 0)));

        let batch: EventBatch<f64> = EventBatch::default().add(Progress::Complete);
        let ctx = EngineContext {
            elapsed: Duration::new(6, 0),
            ..Default::default()
        };

        assert!(matches!(
            stack.decide(&batch, &ctx),
            EngineAction::Stop(crate::Termination::Timeout)
        ))
    }

    #[test]
    fn timeout_policy_continues_for_durations_less_than_limit() {
        let mut stack = PolicyStack::<f64>::new().add(TimeoutPolicy::new(Duration::new(5, 0)));

        let batch: EventBatch<f64> = EventBatch::default().add(Progress::Complete);
        let ctx = EngineContext {
            elapsed: Duration::new(4, 0),
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Continue))
    }
}
