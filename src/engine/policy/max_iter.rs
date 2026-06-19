//! Iteration budget termination policy.
//!
//! This policy enforces a hard upper bound on the number of solver
//! iterations.
//!
//! # Behaviour
//!
//! - If `context.iter > max_iters`, returns
//!   [`EngineAction::Stop(Termination::ExceededMaxIterations)`].
//! - Otherwise returns [`EngineAction::Continue`].
//!
//! # Design notes
//!
//! This is a safety policy intended to guarantee termination even when
//! convergence criteria are not met.
//!
//! It should generally be included in all production policy stacks.
//!
//! The check is strict (`>`), meaning the solver is allowed to reach exactly
//! `max_iters` before stopping is triggered on the next decision cycle.
use super::EnginePolicy;

use crate::engine::{EngineAction, EngineContext, EventBatch};

pub struct MaxIterationPolicy {
    max_iters: usize,
}

impl MaxIterationPolicy {
    pub fn new(max_iters: usize) -> Self {
        Self { max_iters }
    }
}

impl<F> EnginePolicy<F> for MaxIterationPolicy {
    fn decide(&mut self, _batch: &EventBatch<F>, context: &EngineContext) -> EngineAction {
        if context.iter > self.max_iters {
            return EngineAction::Stop(crate::Termination::ExceededMaxIterations);
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
    fn max_iteration_policy_terminates_when_iter_exceeds_limit() {
        let mut stack = PolicyStack::<f64>::new().add(MaxIterationPolicy::new(100));

        let batch: EventBatch<f64> = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 101,
            ..Default::default()
        };

        assert!(matches!(
            stack.decide(&batch, &ctx),
            EngineAction::Stop(crate::Termination::ExceededMaxIterations)
        ))
    }

    #[test]
    fn max_iteration_policy_does_not_terminate_when_iter_is_less_than_limit() {
        let mut stack = PolicyStack::<f64>::new().add(MaxIterationPolicy::new(100));

        let batch: EventBatch<f64> = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 99,
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Continue))
    }
}
