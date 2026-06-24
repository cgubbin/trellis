use crate::result::{EngineOutput, EngineOutputWithSnapshot};
use crate::state::{State, UserState};
use crate::TrellisFloat;

/// Unified result type returned by the engine.
///
/// This captures both:
/// - successful execution paths (`EngineEngineOutput`)
/// - exceptional failure paths (`EngineFailure`)
///
/// The separation is intentional:
/// - `EngineSuccess` = controlled, expected termination
/// - `EngineFailure` = unexpected error during execution
pub type EngineResult<O, S, E> = Result<EngineOutput<O, S>, EngineFailure<S, E>>;

pub(super) type InternalEngineResult<O, S, E> =
    Result<EngineOutput<O, S>, InternalEngineFailure<E>>;

pub type EngineResultWithSnapshot<O, S, E> =
    Result<EngineOutputWithSnapshot<O, S>, EngineFailure<S, E>>;

#[derive(thiserror::Error, Debug)]
pub enum EngineFailure<S, E>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    /// A failure originating from the user-defined procedure.
    ///
    /// This represents an *exceptional error path*, not a normal termination.
    /// It indicates that the solver logic itself could not complete a step
    /// or finalisation phase.
    ///
    /// The included `State` is a snapshot of the engine at the point of failure
    /// and can be used for debugging or checkpoint recovery.
    #[error("error in underlying procedure: {error}")]
    Procedure {
        /// The underlying procedure error.
        error: E,

        /// Snapshot of the solver state at the time of failure.
        state: State<S>,
    },
}

impl<S, E> EngineFailure<S, E>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub(super) fn from_internal(internal: InternalEngineFailure<E>, state: State<S>) -> Self {
        EngineFailure::Procedure {
            error: internal.0,
            state,
        }
    }
}

pub(super) struct InternalEngineFailure<E>(E);

impl<E> InternalEngineFailure<E> {
    pub(super) fn new(error: E) -> Self {
        Self(error)
    }
}
