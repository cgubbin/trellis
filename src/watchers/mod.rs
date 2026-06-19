use crate::engine::EngineStage;
use crate::progress::ProgressRow;
use crate::state::{State, UserState};
use crate::Termination;

mod checkpoint;
mod csv_file;
mod plot;
mod tracing;

use std::sync::{Arc, Mutex};

pub struct ObservationContext {
    pub iteration: usize,
    pub termination: Option<Termination>,
    pub stage: EngineStage,
}

pub trait ProgressObserver<F>: Send + Sync {
    fn observe(&self, progress: ProgressRow<F>);
    fn should_observe(&self, stage: EngineStage) -> bool {
        true
    }
}

pub struct ProgressObservers<F> {
    inner: Vec<(Arc<Mutex<dyn ProgressObserver<F>>>, Frequency)>,
}

impl<F> ProgressObservers<F> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn attach(&mut self, observer: Arc<Mutex<dyn ProgressObserver<F>>>, frequency: Frequency) {
        self.inner.push((observer, frequency))
    }
}

pub trait StateObserver<S: UserState>: Send + Sync {
    fn observe(&self, ident: &'static str, state: &State<S>, ctx: &ObservationContext);
    fn should_observe(&self, stage: EngineStage) -> bool {
        true
    }
}

pub struct StateObservers<S: UserState> {
    inner: Vec<(Arc<Mutex<dyn StateObserver<S>>>, Frequency)>,
}

impl<S: UserState> StateObservers<S> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    pub fn attach(&mut self, observer: Arc<Mutex<dyn StateObserver<S>>>, frequency: Frequency) {
        self.inner.push((observer, frequency))
    }
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
            Frequency::Every(n) => iteration.is_multiple_of(*n),
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

impl<S: UserState> Observers<S> {
    pub fn from_parts(progress: ProgressObservers<S::Float>, state: StateObservers<S>) -> Self {
        Self { progress, state }
    }
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
                // TODO: Unwrap should be handled correctly
                let obs = obs.lock().unwrap();
                if obs.should_observe(ctx.stage) {
                    obs.observe(progress.clone());
                }
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
            // Emit if the frequency of the observer is valid
            //
            // Checkpoints should always emit when called as their emission is policy led, so the
            // check is overridden at a checkpoint.
            if freq.should_emit(ctx.iteration, is_exit) | (ctx.stage == EngineStage::Checkpoint) {
                // TODO: Unwrap should be handled correctly
                let obs = obs.lock().unwrap();
                if obs.should_observe(ctx.stage) {
                    obs.observe(ident, state, ctx);
                }
            }
        }
    }
}
