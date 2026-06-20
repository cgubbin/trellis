use crate::engine::EngineSignal;
use crate::progress::Progress;
use crate::state::StateView;
use crate::watchers::Observe;

use num_traits::float::FloatCore;
use std::sync::Mutex;

pub struct MetricsObserver<F> {
    inner: Mutex<MetricsState<F>>,
}

/// Aggregates run-level statistics into a final summary.
///
/// This observer is useful for post-run analysis and benchmarking.
#[derive(Clone)]
pub struct MetricsState<F> {
    best_metric: Option<F>,
    last_metric: Option<F>,

    best_absolute: Option<F>,
    best_relative: Option<F>,

    iterations: usize,
}

impl<F> Default for MetricsState<F> {
    fn default() -> Self {
        Self {
            best_metric: None,
            last_metric: None,
            best_absolute: None,
            best_relative: None,
            iterations: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunSummary<F> {
    pub iterations: usize,
    pub best_metric: Option<F>,
    pub last_metric: Option<F>,
    pub best_absolute: Option<F>,
    pub best_relative: Option<F>,
}

impl<F> MetricsObserver<F>
where
    F: Copy + PartialOrd,
{
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(MetricsState::default()),
        }
    }

    pub fn summary(self) -> RunSummary<F> {
        let inner = self.inner.lock().unwrap();
        RunSummary {
            iterations: inner.iterations,
            best_metric: inner.best_metric,
            last_metric: inner.last_metric,
            best_absolute: inner.best_absolute,
            best_relative: inner.best_relative,
        }
    }
}

impl<S> Observe<S> for MetricsObserver<S::Float>
where
    S: crate::UserState,
    S::Float: FloatCore + Send + Sync,
{
    fn observe(&self, _: &'static str, state: StateView<'_, S>, event: &EngineSignal<S::Float>) {
        let mut inner = self.inner.lock().unwrap();
        if let EngineSignal::Progress(progress) = event {
            inner.iterations = state.iteration();

            match progress {
                Progress::Measure(measure) => {
                    inner.last_metric = Some(*measure);

                    inner.best_metric = match inner.best_metric {
                        Some(best) if best <= *measure => Some(best),
                        _ => Some(*measure),
                    };
                }

                Progress::Report {
                    diagnostics,
                    measure,
                } => {
                    inner.last_metric = Some(*measure);

                    inner.best_metric = match inner.best_metric {
                        Some(best) if best <= *measure => Some(best),
                        _ => Some(*measure),
                    };

                    if let Some(a) = diagnostics.absolute_error {
                        inner.best_absolute = Some(inner.best_absolute.map_or(a, |b| b.min(a)));
                    }
                    if let Some(r) = diagnostics.relative_error {
                        inner.best_relative = Some(inner.best_relative.map_or(r, |b| b.min(r)));
                    }
                }

                Progress::Complete => {}
            }
        }
    }
}
