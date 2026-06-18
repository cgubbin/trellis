use crate::engine::EngineStage;
use crate::progress::ProgressRow;
use crate::state::{State, UserState};
use crate::Termination;

mod csv_file;
mod plot;
mod tracing;

use std::sync::Arc;

pub struct ObservationContext {
    pub iteration: usize,
    pub termination: Option<Termination>,
    pub stage: EngineStage,
}

pub trait ProgressObserver<F>: Send + Sync {
    fn observe(&self, progress: ProgressRow<F>);
}

pub struct ProgressObservers<F> {
    inner: Vec<(Arc<dyn ProgressObserver<F>>, Frequency)>,
}

pub trait StateObserver<S: UserState>: Send + Sync {
    fn observe(&self, ident: &'static str, state: &State<S>, ctx: &ObservationContext);
}

pub struct StateObservers<S: UserState> {
    inner: Vec<(Arc<dyn StateObserver<S>>, Frequency)>,
}

#[derive(Copy, Clone, Debug)]
pub enum Frequency {
    Always,
    Every(usize),
    OnExit,
    Never,
}

impl Frequency {
    pub fn should_emit(&self, iteration: usize, is_exit: bool) -> bool {
        match self {
            Frequency::Always => true,
            Frequency::Every(n) => iteration % n == 0,
            Frequency::OnExit => is_exit,
            Frequency::Never => false,
        }
    }
}

pub struct Observers<S>
where
    S: UserState,
{
    progress: ProgressObservers<S::Float>,
    state: StateObservers<S>,
}

impl<S> Observers<S>
where
    S: UserState,
{
    pub fn observe_progress(
        &self,
        ident: &'static str,
        progress: ProgressRow<S::Float>,
        ctx: &ObservationContext,
        is_exit: bool,
    ) {
        for (obs, freq) in &self.progress.inner {
            if freq.should_emit(ctx.iteration, is_exit) {
                obs.observe(progress.clone());
            }
        }
    }

    pub fn observe_state(
        &self,
        ident: &'static str,
        state: &State<S>,
        ctx: &ObservationContext,
        is_exit: bool,
    ) {
        for (obs, freq) in &self.state.inner {
            if freq.should_emit(ctx.iteration, is_exit) {
                obs.observe(ident, state, ctx);
            }
        }
    }
}
