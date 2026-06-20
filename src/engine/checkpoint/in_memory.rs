use std::sync::{Arc, Mutex};

use crate::engine::checkpoint::{Checkpoint, CheckpointBackend, CheckpointView};

#[derive(Clone, Default)]
pub struct InMemoryCheckpointStore<SN, F> {
    pub saved: Arc<Mutex<Vec<Checkpoint<SN, F>>>>,
}

impl<SN, F> InMemoryCheckpointStore<SN, F> {
    pub fn new() -> Self {
        Self {
            saved: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn saved_count(&self) -> usize {
        self.saved.lock().unwrap().len()
    }
}

impl<SN, F> CheckpointBackend<SN, F> for InMemoryCheckpointStore<SN, F>
where
    SN: Clone + Send + Sync,
    F: Clone + Send + Sync,
{
    fn save(
        &self,
        checkpoint: CheckpointView<'_, SN, F>,
    ) -> Result<(), crate::engine::checkpoint::CheckpointError> {
        dbg!("saving");
        self.saved.lock().unwrap().push(Checkpoint::new(checkpoint));
        dbg!("saved");
        Ok(())
    }

    fn load(
        &self,
    ) -> Result<Option<Checkpoint<SN, F>>, crate::engine::checkpoint::CheckpointError> {
        Ok(self.saved.lock().unwrap().last().cloned())
    }
}
