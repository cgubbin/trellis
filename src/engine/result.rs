use crate::result::Output;
use crate::state::{State, UserState};
use crate::{Termination, TrellisFloat};

/// Unified result type returned by the engine.
///
/// This captures both:
/// - successful execution paths (`EngineOutput`)
/// - exceptional failure paths (`EngineFailure`)
///
/// The separation is intentional:
/// - `EngineOutput` = controlled, expected termination
/// - `EngineFailure` = unexpected error during execution
pub type EngineResult<O, S, E> = Result<EngineOutput<O, S>, EngineFailure<E, S>>;

/// Result of a successful or cleanly terminated engine run.
///
/// This type represents *completed execution*, meaning the engine reached a
/// well-defined stopping point either via convergence or via a controlled
/// termination condition (timeout, stagnation, cancellation, etc.).
///
/// The included [`Output`] always contains the final solver state snapshot
/// and any user-defined result data produced by the procedure.
pub enum EngineOutput<O, S>
where
    S: UserState,
{
    /// The solver successfully converged according to its configured criteria.
    Success(Output<O, S>),

    /// The solver terminated before convergence due to a policy decision.
    ///
    /// This includes cases such as:
    /// - cancellation
    /// - timeout
    /// - stagnation
    /// - exceeding iteration limits
    ///
    /// The output is still valid and may be inspected, but should not be
    /// interpreted as a converged solution.
    Terminated {
        /// The reason execution stopped.
        termination: Termination,

        /// Final output snapshot at the point of termination.
        output: Output<O, S>,
    },
}

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
