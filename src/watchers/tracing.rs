use num_traits::float::FloatCore;
use tracing::{debug, info, trace, Level, Value};

use crate::watchers::{ObservationError, Observer, Stage};
use crate::{State, UserState};

/// A logger using the tracing crate
#[derive(Clone)]
pub struct Tracer {
    /// the logger
    level: Level,
}

impl Tracer {
    pub fn new(level: Level) -> Self {
        if matches!(level, Level::ERROR | Level::WARN) {
            panic!("we won't emit non-error messages at ERROR or WARN...");
        }
        Self { level }
    }
}

struct TracingState<I>(I);

impl<S> Observer<State<S>> for Tracer
where
    S: UserState,
    S::Float: FloatCore + Value,
{
    fn observe(&self, ident: &'static str, subject: &State<S>, stage: Stage) {
        match stage {
            Stage::Initialisation => self.observe_initialisation(ident),
            Stage::WrapUp => self.observe_wrap_up(ident),
            Stage::Iteration => self.observe_iteration(subject),
        }
        .unwrap()
    }
}

impl Tracer {
    /// Log basic information about the optimization after initialization.
    fn observe_initialisation(&self, name: &str) -> Result<(), ObservationError> {
        // We need to match on the enum rather than using the `event` macro because the `event`
        // macro requires the event type to be static: it cannot be read from the internal field
        match self.level {
            Level::INFO => info!("initialising: {}", name),
            Level::DEBUG => debug!("initialising: {}", name),
            Level::TRACE => trace!("initialising: {}", name),
            _ => unreachable!(
                "constructor should not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }

    fn observe_wrap_up(&self, name: &str) -> Result<(), ObservationError> {
        match self.level {
            Level::INFO => info!("wrap up: {}", name),
            Level::DEBUG => debug!("wrap up: {}", name),
            Level::TRACE => trace!("wrap up: {}", name),
            _ => unreachable!(
                "constructor should not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }

    fn observe_iteration<S>(&self, state: &State<S>) -> Result<(), ObservationError>
    where
        S: UserState,
        S::Float: FloatCore + Value,
    {
        match self.level {
            Level::INFO => info!(
                iteration = state.runtime.iteration(),
                best = state.convergence.best(),
                current = state.convergence.current(),
                since_best = state.iterations_since_best(),
            ),
            Level::DEBUG => debug!(
                iteration = state.runtime.iteration(),
                best = state.convergence.best(),
                current = state.convergence.current(),
                since_best = state.iterations_since_best(),
            ),
            Level::TRACE => trace!(
                iteration = state.runtime.iteration(),
                best = state.convergence.best(),
                current = state.convergence.current(),
                since_best = state.iterations_since_best(),
            ),
            _ => unreachable!(
                "constructor does not allow warn or error level events for non-error messages"
            ),
        };
        Ok(())
    }
}
