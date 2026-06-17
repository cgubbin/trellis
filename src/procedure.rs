use crate::{CancellationGuard, Problem};

/// Trait implemented by all problems solveable by `Trellis`
///
/// A procedure defines the core loop of the solver. Typically we would write a for loop,
/// consisting of an initialisation step where the procedure is arranged, a procedure carried out
/// on each loop iteration, and a finalisation step prior to return. This trait separates these
/// methods so they can be called by the [`Runner`]
pub trait Procedure {
    /// The error associated with the problem
    type Error: std::error::Error + 'static;
    /// The type returned to the caller.
    ///
    /// Trellis defines a data-rich [`Output`], which can be constructed from the procedure, and
    /// internal state. In some circumstances it may be appropriate to return this type to the
    /// caller. In other circumstances it may be preferential to bury this complexity, returning
    /// the caller a custom datatype.
    type Output;

    type Problem;
    type State;

    /// An identifier for the procedure.
    ///
    /// This identifier is printed in tracing logs
    const NAME: &'static str;
    /// Initialisation.
    ///
    /// This step prepares the state object for the main procedure loop.
    fn initialise(
        &mut self,
        _problem: &mut Problem<Self::Problem>,
        state: Self::State,
    ) -> Result<Self::State, Self::Error> {
        Ok(state)
    }
    /// One iteration of the core algorithm
    fn step(
        &mut self,
        problem: &mut Problem<Self::Problem>,
        state: Self::State,
        guard: CancellationGuard<'_>,
    ) -> Result<Self::State, Self::Error>;

    fn is_finished(&mut self, state: Self::State) -> bool;

    /// Converts the internal state to the user-facing return datatype
    fn finalise(
        &mut self,
        problem: &mut Problem<Self::Problem>,
        state: Self::State,
    ) -> Result<Self::Output, Self::Error>;
}
