//! Termination reasons for solver execution.
//!
//! This module defines the canonical set of reasons a solver may stop execution.
//! These values are produced by the engine’s policy layer and propagated to the
//! final `EngineOutput`.
//!
//! The enum is intentionally small and closed: it represents *semantic outcomes*
//! rather than implementation-specific events.
//!
//! In general:
//! - `Converged` is the only *successful* termination.
//! - All other variants represent controlled early termination or failure modes.
//!
//! These values are stable identifiers suitable for logging, serialization,
//! checkpoint metadata, and downstream analysis.
//! Module for abstractions about the state of a solver, and reasons why a solver may have
//! terminated.

use serde::{Deserialize, Serialize};

/// Canonical reasons why a solver terminated.
///
/// This type is produced by the engine policy layer and recorded in the final
/// engine output.
///
/// It is deliberately coarse-grained: it describes *why execution stopped*,
/// not how the engine reached that decision.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Termination {
    /// Execution was cancelled externally (e.g. user request or cancellation token).
    ///
    /// This is considered a non-analytic termination and does not imply failure
    /// of the solver logic.
    Cancelled,

    /// The solver successfully met its convergence criteria.
    ///
    /// This is the only variant considered a successful termination.
    Converged,

    /// The solver exceeded the maximum allowed number of iterations.
    ///
    /// Typically enforced by `MaxIterationPolicy`.
    ExceededMaxIterations,

    /// The solver failed to make sufficient progress over a bounded window of iterations.
    ///
    /// Typically triggered by stagnation or no-progress policies.
    Stagnated,

    /// The solver exceeded a wall-clock time limit.
    ///
    /// Typically enforced by `TimeoutPolicy`.
    Timeout,
}

impl Termination {
    /// Returns `true` if this termination represents a non-successful exit.
    ///
    /// This is used by the engine to classify outputs into:
    /// - success (`Converged`)
    /// - early termination or failure (all other variants)
    ///
    /// Note:
    /// `Cancelled` is treated as a failure-like termination in this sense,
    /// even though it may be externally induced.
    pub(crate) fn failed(&self) -> bool {
        *self != Self::Converged
    }
}
