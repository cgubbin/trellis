use crate::result::{EngineOutput, EngineOutputWithSnapshot, RunSummary};
use crate::state::{Snapshotable, State, StateView, UserState};
use crate::{Termination, TrellisFloat};

/// Unified result type returned by the engine.
///
/// This captures both:
/// - successful execution paths (`EngineEngineOutput`)
/// - exceptional failure paths (`EngineFailure`)
///
/// The separation is intentional:
/// - `EngineSuccess` = controlled, expected termination
/// - `EngineFailure` = unexpected error during execution
pub type EngineResult<O, S, E> = Result<EngineOutput<O, S>, EngineFailure<E, S>>;

pub(super) type InternalEngineResult<O, S, E> =
    Result<(EngineOutput<O, S>, State<S>), EngineFailure<E, S>>;

pub type EngineResultWithSnapshot<O, S: Snapshotable, E> =
    Result<EngineOutputWithSnapshot<O, S>, EngineFailure<E, S>>;

#[derive(thiserror::Error, Debug)]
pub enum EngineFailure<E, S>
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
    Procedure {
        /// The underlying procedure error.
        error: E,

        /// Snapshot of the solver state at the time of failure.
        state: State<S>,
    },
}
