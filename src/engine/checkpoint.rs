use crate::{
    state::{State, UserState},
    TrellisFloat,
};

use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Checkpoint<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub state: State<S>,
}

pub trait CheckpointStore<S>
where
    S: UserState,
{
    type Error;

    fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), Self::Error>;

    fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error>;
}

pub struct NoCheckpoint;

impl<S> CheckpointStore<S> for NoCheckpoint
where
    S: UserState,
{
    type Error = std::convert::Infallible;

    fn save(&mut self, _: &Checkpoint<S>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error> {
        Ok(None)
    }
}

pub struct MemoryCheckpointStore<S>
where
    S: UserState,
{
    checkpoint: Option<Checkpoint<S>>,
}

impl<S> CheckpointStore<S> for MemoryCheckpointStore<S>
where
    S: UserState,
{
    type Error = std::convert::Infallible;

    fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), Self::Error> {
        self.checkpoint = Some(checkpoint.clone());
        Ok(())
    }

    fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error> {
        Ok(self.checkpoint.clone())
    }
}

pub struct FileCheckpointStore {
    path: PathBuf,
}

impl<S> CheckpointStore<S> for FileCheckpointStore
where
    State<S>: Serialize + DeserializeOwned,
    S: UserState,
{
    type Error = std::convert::Infallible;

    fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error> {
        unimplemented!()
    }
}
