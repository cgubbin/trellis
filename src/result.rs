//! This module defines the canonical output types produced by an [`Engine`] execution.
//!
//! It separates:
//! - the *user-defined result* of a computation
//! - the final solver state
//! - termination metadata (success vs early stop)

use crate::{
    state::{Snapshotable, State, StateView, UserState},
    Termination,
};
use num_traits::float::FloatCore;
use std::fmt;

/// Summary information describing a completed engine run.
#[derive(Clone, Debug)]
pub struct RunSummary<F> {
    /// Final iteration count.
    pub iterations: usize,

    /// Total execution time.
    pub elapsed: std::time::Duration,

    /// Best metric value observed during the run.
    pub best_measure: Option<F>,
}

impl<F> RunSummary<F> {
    pub(crate) fn new<S>(state: StateView<'_, S>) -> RunSummary<F>
    where
        S: UserState<Float = F>,
        F: FloatCore,
    {
        Self {
            iterations: state.iteration(),
            elapsed: state.duration(),
            best_measure: Some(state.best_measure()),
        }
    }
}

/// Result of a completed calculation.
///
/// Returned by [`Engine::run`].
pub struct EngineOutput<R, S>
where
    S: UserState,
{
    /// User-defined result produced by the procedure.
    pub result: R,

    /// Execution summary.
    pub summary: RunSummary<S::Float>,

    /// Reason execution terminated.
    pub termination: Termination,
}

/// Result of a completed calculation including a restart snapshot.
///
/// Returned by [`Engine::run_with_snapshot`].
pub struct EngineOutputWithSnapshot<R, S>
where
    S: UserState + Snapshotable,
{
    /// User-defined result produced by the procedure.
    pub result: R,

    /// Snapshot captured from the final state.
    pub snapshot: S::Snapshot,

    /// Execution summary.
    pub summary: RunSummary<S::Float>,

    /// Reason execution terminated.
    pub termination: Termination,
}

impl<R, S> EngineOutput<R, S>
where
    S: UserState,
{
    pub(crate) fn new(result: R, state: StateView<'_, S>, termination: Termination) -> Self {
        let summary = RunSummary::new(state);
        Self {
            result,
            summary,
            termination,
        }
    }

    pub fn with_snapshot(self, snapshot: S::Snapshot) -> EngineOutputWithSnapshot<R, S>
    where
        S: UserState + Snapshotable,
    {
        EngineOutputWithSnapshot {
            result: self.result,
            summary: self.summary,
            termination: self.termination,
            snapshot,
        }
    }
}

#[derive(thiserror::Error, Debug)]
/// Error returned when engine execution fails during procedure execution.
///
/// This error wraps:
/// - the underlying procedure error (`E`)
/// - optionally a partial output (`O`) representing the last known state
///
/// This is useful for partial recovery scenarios where:
/// - execution failed mid-run
/// - but the solver state is still meaningful
pub struct TrellisError<O, E> {
    #[source]
    pub error: E,

    /// Optional partial result produced before failure.
    pub result: Option<O>,
}

impl<O, E: ::std::fmt::Debug> ::std::fmt::Display for TrellisError<O, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Trellis error: {:?}", self.error)
    }
}
