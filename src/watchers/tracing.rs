use num_traits::float::FloatCore;
use tracing::{debug, info, trace, Level, Value};

use crate::watchers::{EngineStage, ObservationContext, StateObserver};
use crate::{State, UserState};

#[derive(Clone)]
pub struct Tracer {
    level: Level,
}

impl Tracer {
    pub fn new(level: Level) -> Self {
        if matches!(level, Level::ERROR | Level::WARN) {
            panic!("Tracer only supports INFO/DEBUG/TRACE levels");
        }
        Self { level }
    }

    fn initialisation(&self, ident: &str) {
        match self.level {
            Level::INFO => info!("initialising: {}", ident),
            Level::DEBUG => debug!("initialising: {}", ident),
            Level::TRACE => trace!("initialising: {}", ident),
            _ => unreachable!(),
        }
    }

    fn wrap_up(&self, ident: &str) {
        match self.level {
            Level::INFO => info!("wrap up: {}", ident),
            Level::DEBUG => debug!("wrap up: {}", ident),
            Level::TRACE => trace!("wrap up: {}", ident),
            _ => unreachable!(),
        }
    }

    fn iteration<S>(&self, state: &State<S>)
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
            _ => unreachable!(),
        }
    }
}

impl<S> StateObserver<S> for Tracer
where
    S: UserState,
    S::Float: FloatCore + Value,
{
    fn observe(&self, ident: &'static str, state: &State<S>, ctx: &ObservationContext) {
        let res = match ctx.stage {
            EngineStage::Initialisation => self.initialisation(ident),
            EngineStage::WrapUp => self.wrap_up(ident),
            EngineStage::Iteration => self.iteration(state),
        };

        // if let Err(e) = res {
        //     eprintln!("Tracer observation error: {e:?}");
        // }
    }
}
