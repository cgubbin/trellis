//! Module for abstractions about the state of a solver, and reasons why a solver may have
//! terminated.

use serde::{Deserialize, Serialize};

/// Causes for termination of a solver
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Termination {
    /// The caller has manually terminated the process
    Cancelled,
    /// The solver has converged to the requested tolerance
    Converged,
    /// The solver has exceeded the maximum allowable iterations
    ExceededMaxIterations,
    /// The solver has exceeded the permitted patience without improvement
    Stagnated,
    /// The solver has run for more than the permitted time
    Timeout,
}

impl Termination {
    pub(crate) fn failed(&self) -> bool {
        *self != Self::Converged
    }
}
