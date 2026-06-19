use crate::{
    state::{State, UserState},
    TrellisFloat,
};

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

mod json;

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("filesystem error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    version: u32,
    state: State<S>,
    timestamp: SystemTime,
}

impl<S> Checkpoint<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub fn new(state: &State<S>) -> Self {
        Self {
            version: 1,
            state: state.clone(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn into_state(self) -> State<S> {
        self.state
    }

    pub fn state(&self) -> &State<S> {
        &self.state
    }
}

pub trait CheckpointStore<S>: Send + Sync
where
    S: UserState,
{
    fn save(&self, checkpoint: &Checkpoint<S>) -> Result<(), CheckpointError>;

    fn load(&self) -> Result<Option<Checkpoint<S>>, CheckpointError>;
}

pub struct NoCheckpoint;

impl<S> CheckpointStore<S> for NoCheckpoint
where
    S: UserState,
{
    fn save(&self, _: &Checkpoint<S>) -> Result<(), CheckpointError> {
        Ok(())
    }

    fn load(&self) -> Result<Option<Checkpoint<S>>, CheckpointError> {
        Ok(None)
    }
}
