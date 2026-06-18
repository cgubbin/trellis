use crate::{State, TrellisFloat, UserState};

#[derive(Debug)]
pub struct Checkpoint<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub state: State<S>,
}

// pub trait PersistCheckpoint {
//     fn save(&self, checkpoint: &Checkpoint<Self::State>);
// }
