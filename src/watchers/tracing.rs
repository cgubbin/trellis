use num_traits::float::FloatCore;
use tracing::Level;

use crate::engine::{EngineSignal, Termination};
use crate::progress::Progress;
use crate::state::{StateView, UserState};
use crate::watchers::Observe;

macro_rules! log_at_level {
    ($level:expr, $($arg:tt)*) => {{
        match $level {
            tracing::Level::ERROR => tracing::error!($($arg)*),
            tracing::Level::WARN => tracing::warn!($($arg)*),
            tracing::Level::INFO => tracing::info!($($arg)*),
            tracing::Level::DEBUG => tracing::debug!($($arg)*),
            tracing::Level::TRACE => tracing::trace!($($arg)*),
        }
    }};
}

/// Structured tracing observer for engine execution.
///
/// This observer emits lifecycle and progress events using the `tracing`
/// ecosystem. It is designed to remain thin and not interpret numerical
/// semantics beyond formatting.
#[derive(Clone)]
pub struct Tracer {
    level: Level,
}

impl Tracer {
    /// Create a new tracer with a base logging level.
    ///
    /// Only INFO, DEBUG, and TRACE are supported. ERROR/WARN are rejected
    /// because this observer is not intended for failure signaling.
    pub fn new(level: Level) -> Self {
        if matches!(level, Level::ERROR | Level::WARN) {
            panic!("Tracer only supports INFO, DEBUG, TRACE levels");
        }

        Self { level }
    }

    fn lifecycle(&self, ident: &str, event_name: &str) {
        log_at_level!(
            self.level,
            target: "engine.lifecycle",
            ident = ident,
            event = event_name,
        );
    }

    fn termination(&self, ident: &str, termination: Termination) {
        log_at_level!(
            self.level,
            target: "engine.termination",
            ident = ident,
            ?termination
        );
    }

    fn progress<S>(&self, state: StateView<'_, S>, progress: &Progress<S::Float>)
    where
        S: UserState,
        S::Float: FloatCore + tracing::Value,
    {
        match progress {
            Progress::Measure(value) => {
                log_at_level!(
                    self.level,
                    target: "engine.progress",
                    kind = "metric",
                    iteration = state.iteration(),
                    value = *value
                );
            }

            Progress::Report {
                measure,
                diagnostics,
            } => {
                log_at_level!(
                    self.level,
                    target: "engine.progress",
                    kind = "report",
                    iteration = state.iteration(),
                    measure = *measure,
                    ?diagnostics
                );
            }

            Progress::Complete => {
                log_at_level!(
                    self.level,
                    target: "engine.progress",
                    kind = "complete",
                    iteration = state.iteration()
                );
            }
        }
    }
}

impl<S> Observe<S> for Tracer
where
    S: UserState,
    S::Float: FloatCore + tracing::Value,
{
    fn observe(
        &self,
        ident: &'static str,
        state: StateView<'_, S>,
        event: &EngineSignal<S::Float>,
    ) {
        match event {
            EngineSignal::Initialised => {
                self.lifecycle(ident, "initialised");
            }

            EngineSignal::CheckpointSaved => {
                self.lifecycle(ident, "checkpoint_saved");
            }

            EngineSignal::CheckpointRequested(_) => {
                self.lifecycle(ident, "checkpoint_requested");
            }

            EngineSignal::Termination(reason) => {
                self.termination(ident, *reason);
            }

            EngineSignal::Progress(progress) => {
                self.progress(state, progress);
            }
        }
    }
}
