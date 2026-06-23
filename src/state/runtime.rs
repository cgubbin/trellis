use std::time::Duration;

/// Runtime bookkeeping for a single engine execution.
///
/// This type is intentionally minimal and **contains no semantic control logic**
/// (no termination, no policy decisions, no convergence state).
///
/// It only tracks:
/// - iteration count
/// - wall-clock duration
///
/// These values are purely observational and are used by:
/// - policies (via `EngineContext`)
/// - diagnostics / summaries
/// - observers / logging
/// - checkpoint metadata
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RuntimeState {
    iter: usize,
    time: Duration,
}

impl RuntimeState {
    /// Creates a fresh runtime state at iteration 0 and zero duration.
    pub(crate) fn new() -> Self {
        Self {
            iter: 0,
            time: Duration::new(0, 0),
        }
    }

    /// Returns the current iteration index.
    ///
    /// This is incremented once per engine step.
    pub fn iteration(&self) -> usize {
        self.iter
    }

    /// Increments the iteration counter by one.
    ///
    /// This should only be called by the engine loop.
    pub fn increment_iteration(&mut self) {
        self.iter += 1;
    }

    /// Returns the total elapsed execution time recorded by the engine.
    ///
    /// Note: this value is *assigned* via `record_duration`, not automatically
    /// computed, to allow flexibility in timing strategies (monotonic clock,
    /// external time source, deterministic replay, etc.).
    pub fn duration(&self) -> Duration {
        self.time
    }

    /// Updates the recorded execution duration.
    ///
    /// Typically set once per iteration in the engine loop.
    pub fn record_duration(&mut self, duration: Duration) {
        self.time = duration;
    }
}
