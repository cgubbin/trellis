use std::{
    fs::{create_dir_all, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::{
    engine::checkpoint::{Checkpoint, CheckpointError, CheckpointStore},
    UserState,
};

pub struct JsonCheckpointStore {
    path: PathBuf,
}

impl JsonCheckpointStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<S> CheckpointStore<S> for JsonCheckpointStore
where
    S: UserState + Serialize + DeserializeOwned,
{
    fn save(&self, checkpoint: &Checkpoint<S>) -> Result<(), CheckpointError> {
        if let Some(parent) = self.path.parent() {
            create_dir_all(parent)?;
        }

        let file = File::create(&self.path)?;

        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &checkpoint)?;

        Ok(())
    }

    fn load(&self) -> Result<Option<Checkpoint<S>>, CheckpointError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.path)?;

        let reader = BufReader::new(file);

        let checkpoint = serde_json::from_reader(reader)?;

        Ok(Some(checkpoint))
    }
}
