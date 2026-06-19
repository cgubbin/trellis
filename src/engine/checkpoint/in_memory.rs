use std::sync::{Arc, Mutex};

use crate::engine::checkpoint::{Checkpoint, CheckpointStore};
use crate::state::UserState;

#[derive(Clone, Default)]
pub struct InMemoryCheckpointStore<S: UserState> {
    pub saved: Arc<Mutex<Vec<Checkpoint<S>>>>,
}

impl<S: UserState> InMemoryCheckpointStore<S> {
    pub fn new() -> Self {
        Self {
            saved: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn saved_count(&self) -> usize {
        self.saved.lock().unwrap().len()
    }
}

impl<S> CheckpointStore<S> for InMemoryCheckpointStore<S>
where
    S: UserState + Send,
    <S as UserState>::Float: Send,
{
    fn save(
        &self,
        checkpoint: &Checkpoint<S>,
    ) -> Result<(), crate::engine::checkpoint::CheckpointError> {
        println!("Saving");
        self.saved.lock().unwrap().push(checkpoint.clone());
        println!("Saved");
        Ok(())
    }

    fn load(&self) -> Result<Option<Checkpoint<S>>, crate::engine::checkpoint::CheckpointError> {
        Ok(None)
    }
}
