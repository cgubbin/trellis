use crate::progress::ProgressReport;
use crate::state::{State, UserState};

use num_traits::float::FloatCore;
use std::time::Duration;

mod action;
mod cancellation;
mod complete;
mod composite;
mod max_iter;
mod no_progress;
mod stagnation;
mod target_value;
mod timeout;
mod tolerance;

pub use cancellation::CancellationPolicy;
pub use complete::CompletionPolicy;
pub use composite::{CompositePolicy, PolicyExt};
pub use max_iter::MaxIterationPolicy;
pub use no_progress::NoProgressPolicy;
pub use stagnation::StagnationPolicy;
pub use target_value::TargetValuePolicy;
pub use timeout::TimeoutPolicy;
pub use tolerance::{AbsoluteTolerancePolicy, RelativeTolerancePolicy};

pub(crate) use action::PolicyDecision;

pub trait EnginePolicy<S: UserState> {
    fn next(
        &mut self,
        state: &State<S>,
        progress: ProgressReport<S::Float>,
        cancelled: bool,
    ) -> PolicyDecision;
}

pub struct Policies;

impl Policies {
    pub fn default<S>(max_iter: usize, atol: S::Float) -> impl EnginePolicy<S>
    where
        S: UserState,
    {
        CancellationPolicy
            .and(MaxIterationPolicy::new(max_iter))
            .and(AbsoluteTolerancePolicy::new(atol))
    }

    pub fn optimisation<S>(
        max_iter: usize,
        atol: S::Float,
        stagnation: usize,
    ) -> impl EnginePolicy<S>
    where
        S: UserState,
    {
        CancellationPolicy
            .and(MaxIterationPolicy::new(max_iter))
            .and(AbsoluteTolerancePolicy::new(atol))
            .and(StagnationPolicy::new(stagnation))
    }

    pub fn global_optimisation<S>(
        max_iter: usize,
        target: S::Float,
        stagnation: usize,
    ) -> impl EnginePolicy<S>
    where
        S: UserState,
        <S as UserState>::Float: FloatCore,
    {
        CancellationPolicy
            .and(MaxIterationPolicy::new(max_iter))
            .and(TargetValuePolicy::new(target))
            .and(StagnationPolicy::new(stagnation))
            .and(NoProgressPolicy::new(S::Float::epsilon(), 50))
    }

    pub fn monte_carlo<S>(max_iter: usize) -> impl EnginePolicy<S>
    where
        S: UserState,
    {
        CancellationPolicy.and(MaxIterationPolicy::new(max_iter))
    }

    pub fn timed<S>(timeout: Duration) -> impl EnginePolicy<S>
    where
        S: UserState,
    {
        CancellationPolicy.and(TimeoutPolicy::new(timeout))
    }
}
