use super::EngineContext;

use num_traits::float::FloatCore;
use std::time::Duration;

mod cancellation;
mod checkpoint;
mod complete;
mod max_iter;
mod no_progress;
mod stagnation;
mod target_value;
mod timeout;
mod tolerance;

pub use cancellation::CancellationPolicy;
pub use max_iter::MaxIterationPolicy;
pub use no_progress::NoProgressPolicy;
pub use stagnation::StagnationPolicy;
pub use target_value::TargetValuePolicy;
pub use timeout::TimeoutPolicy;
pub use tolerance::AbsoluteTolerancePolicy;

use crate::engine::{EngineAction, EventBatch};

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

impl<F> PolicyStack<F> {
    pub fn decide(&mut self, batch: &EventBatch<F>, ctx: &EngineContext) -> EngineAction {
        let mut checkpoint = false;
        for p in &mut self.policies {
            match p.decide(batch, ctx) {
                EngineAction::Stop(t) => return EngineAction::Stop(t),
                EngineAction::Continue => {}
                EngineAction::Step => {}
                EngineAction::EmitCheckpoint => {
                    checkpoint = true;
                }
            }
        }

        if checkpoint {
            return EngineAction::EmitCheckpoint;
        }

        EngineAction::Step
    }
}

impl<F> PolicyStack<F> {
    pub fn default(max_iter: usize, atol: F) -> PolicyStack<F>
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
