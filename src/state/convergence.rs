use super::{RuntimeState, UpdateData};
use num_traits::float::FloatCore;

#[derive(Clone, Debug)]
pub struct ConvergenceState<F> {
    current: F,
    previous: F,

    best: F,
    previous_best: F,

    last_best_iteration: usize,

    relative_tolerance: F,
    absolute_tolerance: F,
}

impl<F: FloatCore> ConvergenceState<F> {
    pub fn new() -> Self {
        Self {
            current: F::infinity(),
            previous: F::infinity(),

            best: F::infinity(),
            previous_best: F::infinity(),

            last_best_iteration: 0,

            relative_tolerance: F::epsilon(),
            absolute_tolerance: F::epsilon(),
        }
    }

    pub fn record(&mut self, measure: F, iteration: usize) -> bool {
        self.previous = self.current;
        self.current = measure;

        let improved = measure < self.best
            || (measure.is_infinite()
                && self.best.is_infinite()
                && measure.is_sign_positive() == self.best.is_sign_positive());

        if improved {
            self.previous_best = self.best;
            self.best = measure;
            self.last_best_iteration = iteration;
        }

        improved
    }

    pub fn current(&self) -> F {
        self.current
    }

    pub fn best(&self) -> F {
        self.best
    }

    pub fn previous(&self) -> F {
        self.previous
    }

    pub fn previous_best(&self) -> F {
        self.previous_best
    }

    pub fn iterations_since_best(&self, current_iteration: usize) -> usize {
        current_iteration - self.last_best_iteration
    }

    pub fn set_relative_tolerance(&mut self, relative_tolerance: F) {
        self.relative_tolerance = relative_tolerance;
    }

    pub fn set_absolute_tolerance(&mut self, absolute_tolerance: F) {
        self.absolute_tolerance = absolute_tolerance;
    }

    pub fn relative_tolerance(&self) -> F {
        self.relative_tolerance
    }

    pub fn absolute_tolerance(&self) -> F {
        self.absolute_tolerance
    }
}
