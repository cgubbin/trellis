//! This module defines the default output type for a trellis calculation, in addition to the error
//! wrapper.

use crate::{Checkpoint, State, Termination, UserState};
use num_traits::float::FloatCore;
use std::fmt;

pub struct TerminatedOutput<O, S>
where
    S: UserState,
{
    pub termination: Termination,
    pub result: Output<O, S>,
}

/// The output of a calculation
///
/// The calculation output is user defined in the finalise step of the [`Calculation`] trait, but
/// this is presented as a good verbose option in situations where the caller wants granular
/// information about the calculation and its progress. It returns the entire original problem,
/// solver and state object.
pub struct Output<R, S>
where
    S: UserState,
{
    /// The original calculation carried out by `trellis`
    pub result: R,
    /// Solver state after the last iterationn
    pub state: State<S>,
}

impl<R, S> std::fmt::Display for Output<R, S>
where
    R: fmt::Display,
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(termination) = self.state.runtime.termination() {
            use crate::Termination::*;
            match termination {
                Converged => {
                    writeln!(
                        f,
                        "Solver converged after {} iterations",
                        self.state.runtime.iteration()
                    )?;
                    if let Some(duration) = self.state.runtime.duration() {
                        writeln!(f, "Duration {:?}", duration)?;
                    }
                    writeln!(f, "{}", self.result)?;
                }
                Cancelled => {
                    writeln!(
                        f,
                        "Solver cancelled after {} iterations",
                        self.state.runtime.iteration()
                    )?;
                }
                ExceededMaxIterations => {
                    writeln!(
                        f,
                        "Solver exceeded maximum iterations ({})",
                        self.state.runtime.iteration()
                    )?;
                }
                Stagnated => {
                    writeln!(
                        f,
                        "Solver stagnant for ({}) iterations",
                        self.state.iterations_since_best()
                    )?;
                }
                Timeout => {
                    writeln!(
                        f,
                        "Solver timed out for ({:?}) ",
                        self.state.runtime.duration()
                    )?;
                }
            }
        } else {
            writeln!(f, "Solver is still in progress.")?;
        }
        Ok(())
    }
}

impl<R, S> Output<R, S>
where
    S: UserState,
{
    pub(crate) fn new(result: R, state: State<S>) -> Self {
        Self { result, state }
    }
}

#[derive(thiserror::Error, Debug)]
/// An error wrapper for trellis calculations
///
/// The error wraps the underlying error type [`ErrorCause`], which contains information about the
/// reason the calculation failed. In addition it can optionally return the an output from the
/// calculation. This is useful in situations where a failure occured due to running out of
/// iterations, or termination from the parent thread, but the state of the calculation at that
/// point may still contain meaningful information. Maybe the calculation ran out of iterations
/// because it was unable to reach the required tolerance, but is still at convergence?
pub struct TrellisError<O, E> {
    #[source]
    /// The underlying error cause.
    pub error: E,
    /// An optional result which can be extracted by the caller
    pub result: Option<O>,
}

impl<O, E: ::std::fmt::Debug> ::std::fmt::Display for TrellisError<O, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Trellis error: {:?}", self.error)
    }
}
