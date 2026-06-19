use crate::{
    state::{State, UserState},
    TrellisFloat,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Checkpoint<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    version: u32,
    state: State<S>,
    timestamp: u64,
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
            timestamp: 0,
        }
    }
}

// pub trait CheckpointStore<S>
// where
//     S: UserState,
// {
//     type Error;

//     fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), Self::Error>;

//     fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error>;
// }

// pub struct NoCheckpoint;

// impl<S> CheckpointStore<S> for NoCheckpoint
// where
//     S: UserState,
// {
//     type Error = std::convert::Infallible;

//     fn save(&mut self, _: &Checkpoint<S>) -> Result<(), Self::Error> {
//         Ok(())
//     }

//     fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error> {
//         Ok(None)
//     }
// }

// pub struct MemoryCheckpointStore<S>
// where
//     S: UserState,
// {
//     checkpoint: Option<Checkpoint<S>>,
// }

// impl<S> CheckpointStore<S> for MemoryCheckpointStore<S>
// where
//     S: UserState,
// {
//     type Error = std::convert::Infallible;

//     fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), Self::Error> {
//         self.checkpoint = Some(checkpoint.clone());
//         Ok(())
//     }

//     fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error> {
//         Ok(self.checkpoint.clone())
//     }
// }

// pub struct FileCheckpointStore {
//     path: PathBuf,
// }

// impl<S> CheckpointStore<S> for FileCheckpointStore
// where
//     State<S>: Serialize + DeserializeOwned,
//     S: UserState,
// {
//     type Error = std::convert::Infallible;

//     fn save(&mut self, checkpoint: &Checkpoint<S>) -> Result<(), Self::Error> {
//         unimplemented!()
//     }

//     fn load(&mut self) -> Result<Option<Checkpoint<S>>, Self::Error> {
//         unimplemented!()
//     }
// }
