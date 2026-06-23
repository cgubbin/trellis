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
pub type EngineResult<O, S> = Result<EngineOutput<O, S>, EngineFailure<S>>;

pub(super) type InternalEngineResult<O, S> = Result<EngineOutput<O, S>, InternalEngineFailure>;

pub type EngineResultWithSnapshot<O, S> = Result<EngineOutputWithSnapshot<O, S>, EngineFailure<S>>;

#[derive(thiserror::Error, Debug)]
pub enum EngineFailure<S>
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
        error: Box<dyn std::error::Error + Send + Sync>,

        /// Snapshot of the solver state at the time of failure.
        state: State<S>,
    },
}

impl<S> EngineFailure<S>
where
    S: UserState,
    <S as UserState>::Float: TrellisFloat,
{
    pub(super) fn from_internal(internal: InternalEngineFailure, state: State<S>) -> Self {
        EngineFailure::Procedure {
            error: internal.0,
            state,
        }
    }
}

pub(super) struct InternalEngineFailure(Box<dyn std::error::Error + Send + Sync>);

impl InternalEngineFailure {
    pub(super) fn new<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self(Box::new(error))
    }
}
