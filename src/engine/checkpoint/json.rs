use std::{
    fs::{create_dir_all, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Serialize};

use crate::engine::checkpoint::{Checkpoint, CheckpointBackend, CheckpointError, CheckpointView};

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

impl<SN, F> CheckpointBackend<SN, F> for JsonCheckpointStore
where
    SN: Serialize + DeserializeOwned,
    F: Serialize + DeserializeOwned,
{
    fn save(&self, checkpoint: CheckpointView<'_, SN, F>) -> Result<(), CheckpointError> {
        if let Some(parent) = self.path.parent() {
            create_dir_all(parent)?;
        }

        let file = File::create(&self.path)?;

        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &checkpoint)?;

        Ok(())
    }

    fn load(&self) -> Result<Option<Checkpoint<SN, F>>, CheckpointError> {
        if !self.path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.path)?;

        let reader = BufReader::new(file);

        let checkpoint = serde_json::from_reader(reader)?;

        Ok(Some(checkpoint))
    }
}
