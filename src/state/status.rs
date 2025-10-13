//! Module for abstractions about the state of a solver, and reasons why a solver may have
//! terminated.

use serde::{Deserialize, Serialize};

/// The status of the solver
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Default)]
pub enum Status {
    /// A solver can either be [`NotTerminated`]
    #[default]
    NotTerminated,
    /// Or the solver can be terminated for [`Cause`]
    Terminated(Cause),
}

/// Causes for termination of a solver
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Cause {
    /// The caller has manually terminated the process with ctrl-C
    ControlC,
    /// A parent thread had terminated the process using a [`tokio::CancellationToken`]
    Parent,
    /// The solver has converged to the requested tolerance
    Converged,
    /// The solver has exceeded the maximum allowable iterations
    ExceededMaxIterations,
}
