use num_traits::float::FloatCore;
use tracing::{debug, info, trace, Level, Value};

use crate::engine::{EngineEvent, Termination};
use crate::progress::Progress;
use crate::state::{StateView, UserState};
use crate::watchers::Observe;

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

    fn lifecycle(&self, message: &str, ident: &str) {
        match self.level {
            Level::INFO => info!("{message}: {ident}"),
            Level::DEBUG => debug!("{message}: {ident}"),
            Level::TRACE => trace!("{message}: {ident}"),
            _ => unreachable!(),
        }
    }

    fn termination(&self, ident: &str, termination: Termination) {
        match self.level {
            Level::INFO => info!(?termination, "terminated: {ident}"),
            Level::DEBUG => debug!(?termination, "terminated: {ident}"),
            Level::TRACE => trace!(?termination, "terminated: {ident}"),
            _ => unreachable!(),
        }
    }

    fn progress<S>(&self, state: StateView<'_, S>, progress: &Progress<S::Float>)
    where
        S: UserState,
        S::Float: FloatCore + Value,
    {
        match progress {
            Progress::ErrorEstimate { absolute, relative } => match self.level {
                Level::INFO => info!(
                    iteration = state.iteration(),
                    current = *absolute,
                    relative = *relative,
                    best = state.best_measure(),
                    since_best = state.iterations_since_best(),
                ),
                Level::DEBUG => debug!(
                    iteration = state.iteration(),
                    current = *absolute,
                    relative = *relative,
                    best = state.best_measure(),
                    since_best = state.iterations_since_best(),
                ),
                Level::TRACE => trace!(
                    iteration = state.iteration(),
                    current = *absolute,
                    relative = *relative,
                    best = state.best_measure(),
                    since_best = state.iterations_since_best(),
                ),
                _ => unreachable!(),
            },

            Progress::Metric { value } => match self.level {
                Level::INFO => info!(iteration = state.iteration(), value = *value,),
                Level::DEBUG => debug!(iteration = state.iteration(), value = *value,),
                Level::TRACE => trace!(iteration = state.iteration(), value = *value,),
                _ => unreachable!(),
            },

            Progress::Complete => match self.level {
                Level::INFO => info!(iteration = state.iteration(), "completion reported"),
                Level::DEBUG => debug!(iteration = state.iteration(), "completion reported"),
                Level::TRACE => trace!(iteration = state.iteration(), "completion reported"),
                _ => unreachable!(),
            },
        }
    }
}

impl<S> Observe<S> for Tracer
where
    S: UserState,
    S::Float: FloatCore + Value,
{
    fn observe(&self, ident: &'static str, state: StateView<'_, S>, event: &EngineEvent<S::Float>) {
        match event {
            EngineEvent::Initialised => {
                self.lifecycle("initialising", ident);
            }

            EngineEvent::CheckpointSaved => {
                self.lifecycle("checkpoint saved", ident);
            }

            EngineEvent::Termination(reason) => {
                self.termination(ident, *reason);
            }

            EngineEvent::Progress(progress) => {
                self.progress(state, progress);
            }
        }
    }
}
