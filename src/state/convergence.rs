//! Convergence tracking state.
//!
//! This module stores the numerical history required by policies,
//! observers, and result reporting.
//!
//! The state records:
//!
//! - current convergence measure
//! - best convergence measure observed so far
//! - objective measures emitted by the procedure
//! - iteration at which the last improvement occurred
//!
//! Convergence information is updated from [`Progress`] signals
//! produced during execution.
//!
//! This structure is internal engine state and is not intended to
//! implement convergence logic itself. Termination decisions are
//! handled by engine policies.
use crate::progress::Progress;

use num_traits::float::FloatCore;
use serde::{Deserialize, Serialize};

fn is_improvement<F: FloatCore>(value: F, best: F) -> bool {
    value < best
        || (value.is_infinite()
            && best.is_infinite()
            && value.is_sign_positive() == best.is_sign_positive())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeasureHistory<F> {
    current: F,
    previous: F,

    best: F,
    previous_best: F,
}

impl<F: FloatCore> Default for MeasureHistory<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: FloatCore> MeasureHistory<F> {
    fn new() -> Self {
        Self {
            current: F::infinity(),
            previous: F::infinity(),
            best: F::infinity(),
            previous_best: F::infinity(),
        }
    }

    pub fn observe(&mut self, value: F) -> bool {
        self.previous = self.current;
        self.current = value;

        let improved = value < self.best;

        if is_improvement(value, self.best) {
            self.previous_best = self.best;
            self.best = value;
        }

        improved
    }
}

/// Tracks convergence and optimisation progress throughout execution.
///
/// The engine updates this state after every iteration using the
/// emitted [`Progress`] signals.
///
/// Both error estimates and optimisation measures may be recorded.
/// The stored values can be queried by policies, observers, and
/// output summaries.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConvergenceState<F> {
    measure: MeasureHistory<F>,
    last_best_iteration: usize,
}

impl<F: FloatCore> Default for ConvergenceState<F> {
    fn default() -> Self {
        Self::new()
    }
}

impl<F: FloatCore> ConvergenceState<F> {
    pub fn new() -> Self {
        Self {
            measure: MeasureHistory::new(),
            last_best_iteration: 0,
        }
    }

    pub fn best(&self) -> F {
        self.measure.best
    }

    pub fn current(&self) -> F {
        self.measure.current
    }

    pub fn previous(&self) -> F {
        self.measure.previous
    }

    pub fn previous_best(&self) -> F {
        self.measure.previous_best
    }

    pub fn iterations_since_best(&self, current_iteration: usize) -> usize {
        current_iteration - self.last_best_iteration
    }

    /// Updates convergence tracking from a progress signal.
    ///
    /// Improvement automatically updates the stored best value and
    /// records the iteration at which it occurred.
    pub fn observe(&mut self, progress: &Progress<F>, iteration: usize) {
        match progress {
            Progress::Complete => {}
            Progress::Measure(measure) => {
                let improved = self.measure.observe(*measure);
                if improved {
                    self.last_best_iteration = iteration;
                }
            }
            Progress::Report { measure, .. } => {
                let improved = self.measure.observe(*measure);
                if improved {
                    self.last_best_iteration = iteration;
                }
            }
        }
    }
}
