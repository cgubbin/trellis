use crate::{State, UserState};

pub struct Checkpoint<S>
where
    S: UserState,
{
    pub state: State<S>,
}

// pub trait PersistCheckpoint {
//     fn save(&self, checkpoint: &Checkpoint<Self::State>);
// }
