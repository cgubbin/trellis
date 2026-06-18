use crate::{
    state::{State, UserState},
    TrellisFloat,
};

pub trait CheckpointSink<S: UserState> {
    fn save(&mut self, state: &State<S>);
    fn load(&mut self) -> Option<Checkpoint<S>>;
}

#[derive(Debug)]
pub struct Checkpoint<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub state: State<S>,
}
