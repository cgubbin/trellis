use super::RuntimeState;
use crate::progress::Progress;
use num_traits::float::FloatCore;

#[derive(Clone, Debug)]
pub struct ConvergenceState<F> {
    current: F,
    previous: F,

    best: F,
    previous_best: F,

    last_best_iteration: usize,

    current_metric: F,
    best_metric: F,
    previous_metric: F,
    previous_best_metric: F,
}

impl<F: FloatCore> ConvergenceState<F> {
    pub fn new() -> Self {
        Self {
            current: F::infinity(),
            previous: F::infinity(),

            best: F::infinity(),
            previous_best: F::infinity(),

            last_best_iteration: 0,

            current_metric: F::infinity(),
            best_metric: F::infinity(),

            previous_metric: F::infinity(),
            previous_best_metric: F::infinity(),
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

    pub fn observe(&mut self, progress: Progress<F>, iteration: usize) {
        match progress {
            Progress::Complete => {}
            Progress::ErrorEstimate { absolute, relative } => {
                self.current = absolute;
                if self.current < self.best
                    || (FloatCore::is_infinite(self.current)
                        && FloatCore::is_infinite(self.best)
                        && FloatCore::is_sign_positive(self.current)
                            == FloatCore::is_sign_positive(self.best))
                {
                    std::mem::swap(&mut self.previous_best, &mut self.best);
                    self.best = self.current;
                    self.last_best_iteration = iteration;
                }
            }
            Progress::Metric { value } => {
                self.current_metric = value;
                if self.current_metric < self.best_metric
                    || (FloatCore::is_infinite(self.current_metric)
                        && FloatCore::is_infinite(self.best_metric)
                        && FloatCore::is_sign_positive(self.current_metric)
                            == FloatCore::is_sign_positive(self.best_metric))
                {
                    std::mem::swap(&mut self.previous_best_metric, &mut self.best_metric);
                    self.best_metric = self.current_metric;
                    self.last_best_iteration = iteration;
                }
            }
        }
    }
}
