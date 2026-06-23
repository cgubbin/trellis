use crate::CancellationGuard;

/// Trait implemented by all problems solveable by `Trellis`
///
/// A procedure defines the core loop of the solver. Typically we would write a for loop,
/// consisting of an initialisation step where the procedure is arranged, a procedure carried out
/// on each loop iteration, and a finalisation step prior to return. This trait separates these
/// methods so they can be called by the [`Runner`]
pub trait Procedure<P> {
    /// The type returned to the caller.
    ///
    /// Trellis defines a data-rich [`Output`], which can be constructed from the procedure, and
    /// internal state. In some circumstances it may be appropriate to return this type to the
    /// caller. In other circumstances it may be preferential to bury this complexity, returning
    /// the caller a custom datatype.
    type Output;

    type State;

    /// An identifier for the procedure.
    ///
    /// This identifier is printed in tracing logs
    const NAME: &'static str;
    /// Initialisation.
    ///
    /// This step prepares the state object for the main procedure loop.
    fn initialise(&self, _problem: &mut P, _state: &mut Self::State) {
        ()
    }
    /// One iteration of the core algorithm
    fn step(&self, problem: &mut P, state: &mut Self::State, guard: CancellationGuard<'_>);

    /// Converts the internal state to the user-facing return datatype
    fn finalise(&self, problem: &mut P, state: &Self::State) -> Self::Output;
}

pub trait FallibleProcedure<P> {
    type Error: std::error::Error + 'static + Send + Sync;

    type Output;

    type State;

    /// An identifier for the procedure.
    ///
    /// This identifier is printed in tracing logs
    const NAME: &'static str;

    fn initialise_fallible(
        &self,
        _problem: &mut P,
        _state: &mut Self::State,
    ) -> Result<(), Self::Error>;

    fn step_fallible(
        &self,
        problem: &mut P,
        state: &mut Self::State,
        guard: CancellationGuard<'_>,
    ) -> Result<(), Self::Error>;

    fn finalise_fallible(
        &self,
        problem: &mut P,
        state: &Self::State,
    ) -> Result<Self::Output, Self::Error>;
}

impl<Proc, P> FallibleProcedure<P> for Proc
where
    Proc: Procedure<P>,
{
    const NAME: &'static str = <Proc as Procedure<P>>::NAME;
    type State = <Proc as Procedure<P>>::State;
    type Output = <Proc as Procedure<P>>::Output;
    type Error = std::convert::Infallible;

    fn initialise_fallible(
        &self,
        problem: &mut P,
        state: &mut Self::State,
    ) -> Result<(), Self::Error> {
        Ok(self.initialise(problem, state))
    }

    fn step_fallible(
        &self,
        problem: &mut P,
        state: &mut Self::State,
        guard: CancellationGuard<'_>,
    ) -> Result<(), Self::Error> {
        Ok(self.step(problem, state, guard))
    }

    fn finalise_fallible(
        &self,
        problem: &mut P,
        state: &Self::State,
    ) -> Result<Self::Output, Self::Error> {
        Ok(self.finalise(problem, state))
    }
}
