use crate::{
    engine::{extensions::EngineSink, EngineSignal},
    state::{
        ConvergenceState, RuntimeState, Snapshotable, State, StateRestorer, StateView, UserState,
    },
};

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

mod in_memory;
mod json;

pub use in_memory::InMemoryCheckpointStore;
pub use json::JsonCheckpointStore;

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("filesystem error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

pub struct CheckpointExtension<C> {
    store: C,
}

impl<C> CheckpointExtension<C> {
    pub(crate) fn new(store: C) -> Self {
        Self { store }
    }
}

impl<S, C> EngineSink<S> for CheckpointExtension<C>
where
    S: UserState + Snapshotable,
    C: CheckpointBackend<<S as Snapshotable>::Snapshot, <S as UserState>::Float> + 'static,
{
    fn handle(&mut self, state: StateView<'_, S>, signal: &EngineSignal<<S as UserState>::Float>) {
        if let EngineSignal::CheckpointRequested(_) = signal {
            let checkpoint = CheckpointView::from(state);

            if let Err(e) = self.store.save(checkpoint) {
                eprintln!("error saving checkpoint: {e:?}");
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CheckpointView<'a, SN, F> {
    user: SN,
    runtime: &'a RuntimeState,
    convergence: &'a ConvergenceState<F>,
    version: u32,
    timestamp: SystemTime,
}

impl<'a, S> From<StateView<'a, S>> for CheckpointView<'a, S::Snapshot, <S as UserState>::Float>
where
    S: Snapshotable + UserState,
{
    fn from(state: StateView<'a, S>) -> CheckpointView<'a, S::Snapshot, <S as UserState>::Float> {
        Self {
            user: state.user().snapshot(),
            runtime: state.runtime(),
            convergence: state.convergence(),
            version: 1,
            timestamp: SystemTime::now(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Checkpoint<SN, F> {
    user: SN,
    runtime: RuntimeState,
    convergence: ConvergenceState<F>,
    version: u32,
    timestamp: SystemTime,
}

impl<SN, F> Checkpoint<SN, F> {
    pub fn new(view: CheckpointView<'_, SN, F>) -> Self
    where
        F: Clone,
    {
        Self {
            user: view.user,
            runtime: view.runtime.clone(),
            convergence: view.convergence.clone(),
            version: view.version,
            timestamp: view.timestamp,
        }
    }
}

impl<SN, F> Checkpoint<SN, F> {
    pub fn into_state<S>(self) -> State<S>
    where
        S: StateRestorer<S> + Snapshotable<Snapshot = SN> + UserState<Float = F>,
    {
        State {
            user: S::restore(self.user),
            runtime: self.runtime,
            convergence: self.convergence,
        }
    }
}

pub trait CheckpointBackend<SN, F>: Send + Sync {
    fn save(&self, checkpoint: CheckpointView<'_, SN, F>) -> Result<(), CheckpointError>;

    fn load(&self) -> Result<Option<Checkpoint<SN, F>>, CheckpointError>;
}

pub trait EngineInitializer<S>
where
    S: UserState,
{
    fn try_load(&self) -> Result<Option<State<S>>, CheckpointError>;
}

impl<S, C> EngineInitializer<S> for C
where
    S: UserState + Snapshotable + StateRestorer<S>,
    C: CheckpointBackend<<S as Snapshotable>::Snapshot, <S as UserState>::Float>,
{
    fn try_load(&self) -> Result<Option<State<S>>, CheckpointError> {
        let maybe_checkpoint = self.load()?;

        Ok(maybe_checkpoint.map(|checkpoint| checkpoint.into_state()))
    }
}
