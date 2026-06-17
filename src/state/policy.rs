use super::{ConvergenceState, Progress, RuntimeState, State, UpdateData, UserState};
use crate::{Termination, TrellisFloat};

#[derive(Clone, Debug)]
pub enum StepDecision {
    Continue,
    Terminate(Termination),
    // This policy does not decide, another will handle it
    Pass,
}

#[derive(Default)]
pub struct DefaultConvergencePolicy<S: UserState> {
    _phantom: std::marker::PhantomData<S>,
}

pub trait ConvergencePolicy<S: UserState> {
    type Float: TrellisFloat;

    fn step(
        &mut self,
        state: &mut State<S>,
        progress: Option<Progress<Self::Float>>,
        cancelled: bool,
    ) -> StepDecision;
}

impl<S> ConvergencePolicy<S> for DefaultConvergencePolicy<S>
where
    S: UserState,
    S::Float: TrellisFloat,
{
    type Float = S::Float;

    fn step(
        &mut self,
        state: &mut State<S>,
        progress: Option<Progress<S::Float>>,
        cancelled: bool,
    ) -> StepDecision {
        if cancelled {
            return StepDecision::Terminate(Termination::Cancelled);
        }

        if let Some(progress) = progress {
            match progress {
                Progress::Metric { value } => {
                    state.convergence.record(value, state.runtime.iteration());
                }

                Progress::ErrorEstimate { absolute, .. } => {
                    state
                        .convergence
                        .record(absolute, state.runtime.iteration());
                }
            }
        }

        if state.convergence.current() < state.convergence.absolute_tolerance() {
            state.runtime.terminate(Termination::Converged);
            return StepDecision::Terminate(Termination::Converged);
        }

        if state.runtime.iteration() > state.runtime.max_iterations() {
            state.runtime.terminate(Termination::ExceededMaxIterations);
            return StepDecision::Terminate(Termination::ExceededMaxIterations);
        }

        StepDecision::Continue
    }
}

pub struct CompositePolicy<S, A, B>
where
    S: UserState,
    A: ConvergencePolicy<S>,
    B: ConvergencePolicy<S>,
{
    pub a: A,
    pub b: B,
    _s: std::marker::PhantomData<S>,
}

impl<S, A, B> ConvergencePolicy<S> for CompositePolicy<S, A, B>
where
    S: UserState,
    S::Float: TrellisFloat,
    A: ConvergencePolicy<S, Float = S::Float>,
    B: ConvergencePolicy<S, Float = S::Float>,
{
    type Float = S::Float;

    fn step(
        &mut self,
        state: &mut State<S>,
        progress: Option<Progress<S::Float>>,
        cancelled: bool,
    ) -> StepDecision {
        match self.a.step(state, progress.clone(), cancelled) {
            StepDecision::Continue => return StepDecision::Continue,
            StepDecision::Terminate(t) => return StepDecision::Terminate(t),
            StepDecision::Pass => {}
        }

        self.b.step(state, progress, cancelled)
    }
}

pub trait PolicyExt<S: UserState>: ConvergencePolicy<S> + Sized {
    fn and<P>(self, other: P) -> CompositePolicy<S, Self, P>
    where
        P: ConvergencePolicy<S>,
    {
        CompositePolicy {
            a: self,
            b: other,
            _s: std::marker::PhantomData,
        }
    }
}

impl<S, T> PolicyExt<S> for T
where
    S: UserState,
    T: ConvergencePolicy<S>,
{
}
