use std::fs::File;

use super::Observe;

use crate::{
    engine::{Checkpoint, EngineEvent, EngineStage},
    state::{StateView, UserState},
};

use std::path::PathBuf;

pub struct CheckpointWriter<S> {
    path: PathBuf,
    _marker: std::marker::PhantomData<S>,
}

impl<S> CheckpointWriter<S> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            _marker: Default::default(),
        }
    }
}

pub trait CheckpointStore<S: UserState> {
    fn save(&self, state: Checkpoint<S>) -> Result<(), std::io::Error>;
}

pub struct BinaryCheckpointStore {
    path: PathBuf,
}

impl BinaryCheckpointStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl<S> CheckpointStore<S> for BinaryCheckpointStore
where
    S: UserState + serde::Serialize,
{
    fn save(&self, checkpoint: Checkpoint<S>) -> Result<(), std::io::Error> {
        let file = File::create(&self.path)?;

        bincode::serialize_into(file, &checkpoint).map_err(std::io::Error::other)
    }
}

pub struct JsonCheckpointStore {
    path: PathBuf,
}

impl JsonCheckpointStore {
    pub fn new<P: Into<std::path::PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl<S> CheckpointStore<S> for JsonCheckpointStore
where
    S: UserState + serde::Serialize,
{
    fn save(&self, checkpoint: Checkpoint<S>) -> Result<(), std::io::Error> {
        let file = std::fs::File::create(&self.path)?;

        serde_json::to_writer_pretty(std::io::BufWriter::new(file), &checkpoint)?;

        Ok(())
    }
}

impl JsonCheckpointStore {
    pub fn load<S>(&self) -> Result<Checkpoint<S>, Box<dyn std::error::Error>>
    where
        S: UserState + serde::de::DeserializeOwned,
    {
        let file = std::fs::File::open(&self.path)?;

        Ok(serde_json::from_reader(std::io::BufReader::new(file))?)
    }
}

pub struct CheckpointObserver<STORE> {
    store: STORE,
}

impl<STORE> CheckpointObserver<STORE> {
    pub fn new(store: STORE) -> Self {
        Self { store }
    }
}

impl<S, STORE> Observe<S> for CheckpointObserver<STORE>
where
    S: UserState,
    STORE: CheckpointStore<S> + Send + Sync,
{
    fn observe(
        &self,
        _ident: &'static str,
        state: StateView<'_, S>,
        event: &EngineEvent<S::Float>,
    ) {
        match event {
            EngineEvent::Stage(EngineStage::Checkpoint) => {
                let checkpoint = Checkpoint::new(state);
                let _ = self.store.save(checkpoint);
            }
            _ => {}
        }
    }
}
