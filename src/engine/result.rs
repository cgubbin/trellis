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
pub enum EngineFailure<E, C, S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    Procedure { error: E, state: State<S> },
    Checkpoint { error: C, state: State<S> },
}

pub type EngineResult<O, S, E, C> = Result<EngineOutput<O, S>, EngineFailure<E, C, S>>;
