use crate::result::Output;
use crate::state::{State, UserState};
use crate::{Termination, TrellisFloat};

/// Type alias for the result of running a calculation
pub enum EngineOutput<O, S>
where
    S: UserState,
{
    Success(Output<O, S>),
    Terminated {
        termination: Termination,
        output: Output<O, S>,
    },
}

#[derive(thiserror::Error, Debug)]
pub struct EngineFailure<E, S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub error: E,
    pub state: State<S>,
}

pub type EngineResult<O, S, E> = Result<EngineOutput<O, S>, EngineFailure<E, S>>;
