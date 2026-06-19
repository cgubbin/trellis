//! Engine termination and control policies.
//!
//! Policies inspect the stream of progress events produced by the engine
//! together with the current execution context and decide whether execution
//! should continue, emit a checkpoint, or terminate.
//!
//! Multiple policies may be combined using [`PolicyStack`].
//!
//! # Built-in policies
//!
//! - [`CancellationPolicy`] — terminates when cancellation is requested.
//! - [`MaxIterationPolicy`] — terminates after a fixed number of iterations.
//! - [`AbsoluteTolerancePolicy`] — terminates when the reported error falls
//!   below a tolerance.
//! - [`TargetValuePolicy`] — terminates when an objective reaches a target.
//! - [`StagnationPolicy`] — terminates when no new best value is observed for
//!   a specified number of iterations.
//! - [`NoProgressPolicy`] — terminates when progress falls below a threshold.
//! - [`TimeoutPolicy`] — terminates after a wall-clock duration.
//!
//! # Examples
//!
//! ```ignore
//! let policy = PolicyStack::optimisation(
//!     10_000,
//!     1e-8,
//!     200,
//! );
//! ```
//!
//! Policies are evaluated in order. The first policy requesting termination
//! immediately stops evaluation.
use super::EngineContext;

use num_traits::float::FloatCore;
use std::time::Duration;

mod absolute_tolerance;
mod cancellation;
mod checkpoint;
mod complete;
mod max_iter;
mod no_progress;
mod relative_tolerance;
mod stagnation;
mod target_value;
mod timeout;

pub use absolute_tolerance::AbsoluteTolerancePolicy;
pub use cancellation::CancellationPolicy;
pub use checkpoint::CheckpointPolicy;
pub use complete::CompletionPolicy;
pub use max_iter::MaxIterationPolicy;
pub use no_progress::NoProgressPolicy;
pub use relative_tolerance::RelativeTolerancePolicy;
pub use stagnation::StagnationPolicy;
pub use target_value::TargetValuePolicy;
pub use timeout::TimeoutPolicy;

use crate::engine::{event::CheckpointReason, EngineAction, EventBatch};

pub trait EnginePolicy<F> {
    fn decide(&mut self, batch: &EventBatch<F>, context: &EngineContext) -> EngineAction;
}

pub trait PolicyExt<F>: EnginePolicy<F> + Sized + 'static {
    fn boxed(self) -> Box<dyn EnginePolicy<F>> {
        Box::new(self)
    }
}

impl<F, T> PolicyExt<F> for T where T: EnginePolicy<F> + Sized + 'static {}

pub struct PolicyStack<F> {
    policies: Vec<Box<dyn EnginePolicy<F>>>,
}

impl<F> Default for PolicyStack<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F> PolicyStack<F> {
    pub fn new() -> Self {
        Self { policies: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.policies.is_empty()
    }

    pub fn add<P>(mut self, p: P) -> Self
    where
        P: EnginePolicy<F> + 'static,
    {
        self.policies.push(Box::new(p));
        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        for each in other.policies.into_iter() {
            self.policies.push(each);
        }
        self
    }
}

impl<F> EnginePolicy<F> for PolicyStack<F> {
    fn decide(&mut self, batch: &EventBatch<F>, ctx: &EngineContext) -> EngineAction {
        let mut checkpoint = false;
        for p in &mut self.policies {
            match p.decide(batch, ctx) {
                EngineAction::Stop(t) => return EngineAction::Stop(t),
                EngineAction::Continue => {}
                EngineAction::EmitCheckpoint(_) => {
                    checkpoint = true;
                }
            }
        }

        if checkpoint {
            return EngineAction::EmitCheckpoint(CheckpointReason::Scheduled);
        }

        EngineAction::Continue
    }
}

impl<F> PolicyStack<F> {
    pub fn standard(max_iter: usize, atol: F) -> PolicyStack<F>
    where
        F: FloatCore + 'static,
    {
        PolicyStack::new()
            .add(CancellationPolicy)
            .add(MaxIterationPolicy::new(max_iter))
            .add(AbsoluteTolerancePolicy::new(atol))
    }

    pub fn optimisation(max_iter: usize, atol: F, stagnation: usize) -> PolicyStack<F>
    where
        F: FloatCore + 'static,
    {
        PolicyStack::new()
            .add(CancellationPolicy)
            .add(MaxIterationPolicy::new(max_iter))
            .add(AbsoluteTolerancePolicy::new(atol))
            .add(StagnationPolicy::new(stagnation))
    }

    pub fn global_optimisation(max_iter: usize, target: F, stagnation: usize) -> PolicyStack<F>
    where
        F: FloatCore + 'static,
    {
        PolicyStack::new()
            .add(CancellationPolicy)
            .add(MaxIterationPolicy::new(max_iter))
            .add(TargetValuePolicy::new(target))
            .add(StagnationPolicy::new(stagnation))
            .add(NoProgressPolicy::new(F::epsilon(), 50))
    }

    pub fn monte_carlo(max_iter: usize) -> PolicyStack<F>
    where
        F: 'static,
    {
        PolicyStack::new()
            .add(CancellationPolicy)
            .add(MaxIterationPolicy::new(max_iter))
    }

    pub fn timed(timeout: Duration) -> PolicyStack<F>
    where
        F: 'static,
    {
        PolicyStack::new()
            .add(CancellationPolicy)
            .add(TimeoutPolicy::new(timeout))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::progress::Progress;

    #[test]
    fn empty_stack_continues() {
        let mut stack = PolicyStack::<f64>::new();

        let batch: EventBatch<f64> = EventBatch::new();

        let ctx = EngineContext::default();

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Continue));
    }

    #[test]
    fn checkpoint_request_is_propagated() {
        let mut stack = PolicyStack::<f64>::new()
            .add(CheckpointPolicy::every(10))
            .add(MaxIterationPolicy::new(500));

        let batch: EventBatch<f64> = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 10,
            ..Default::default()
        };

        assert!(matches!(
            stack.decide(&batch, &ctx),
            EngineAction::EmitCheckpoint(_)
        ));
    }

    #[test]
    fn stop_takes_precedence_over_checkpoint() {
        let mut stack = PolicyStack::<f64>::new()
            .add(CheckpointPolicy::every(10))
            .add(MaxIterationPolicy::new(0));

        let batch: EventBatch<f64> = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 10,
            ..Default::default()
        };

        assert!(matches!(stack.decide(&batch, &ctx), EngineAction::Stop(_)));
    }

    #[test]
    fn policy_stack_stop_overrides_all() {
        let mut stack = PolicyStack::new()
            .add(NoProgressPolicy::new(0.1, 10))
            .add(MaxIterationPolicy::new(100));

        let batch = EventBatch::new().add(Progress::Complete);
        let ctx = EngineContext {
            iter: 101,
            ..Default::default()
        };

        let action = stack.decide(&batch, &ctx);

        if let EngineAction::Stop(_) = action {
            assert!(true);
        } else {
            panic!("Stop must dominate all policies");
        }
    }

    #[test]
    fn integration_converges_via_tolerance() {
        let mut stack = PolicyStack::<f64>::optimisation(100, 0.01, 5);

        let ctx = EngineContext::default();

        let batch = EventBatch::new().add(Progress::ErrorEstimate {
            absolute: 0.001,
            relative: 0.2,
        });

        let action = stack.decide(&batch, &ctx);

        assert!(matches!(
            action,
            EngineAction::Stop(crate::Termination::Converged)
        ));
    }

    #[test]
    fn integration_stagnation_overrides_no_progress() {
        let mut stack = PolicyStack::<f64>::new()
            .add(NoProgressPolicy::new(0.01, 3))
            .add(StagnationPolicy::new(5));

        let ctx = EngineContext::default();

        for _ in 0..10 {
            let batch = EventBatch::new().add(Progress::Metric { value: 1.0 });

            let action = stack.decide(&batch, &ctx);

            if let EngineAction::Stop(_) = action {
                break;
            }
        }
    }

    #[test]
    fn integration_timeout_trumps_all() {
        let mut stack = PolicyStack::<f64>::timed(Duration::from_secs(1));

        let batch = EventBatch::new().add(Progress::Complete);

        let ctx = EngineContext {
            elapsed: Duration::from_secs(10),
            ..Default::default()
        };

        assert!(matches!(
            stack.decide(&batch, &ctx),
            EngineAction::Stop(crate::Termination::Timeout)
        ));
    }
}
