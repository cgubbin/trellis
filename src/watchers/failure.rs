use crate::engine::EngineSignal;
use crate::progress::Progress;
use crate::state::StateView;
use crate::watchers::Observe;

use num_traits::float::FloatCore;
use std::sync::Mutex;

#[derive(Default)]
pub struct FailureState<F> {
    last_metric: Option<F>,
    last_iteration: usize,
    last_kind: Option<&'static str>,
}

/// Captures diagnostic snapshots when a run terminates.
///
/// Useful for debugging stagnation, divergence, or premature termination.
#[derive(Default)]
pub struct FailureDiagnostics<F> {
    inner: Mutex<FailureState<F>>,
}

#[derive(Debug, Clone)]
pub struct FailureReport<F> {
    pub last_metric: Option<F>,
    pub last_iteration: usize,
    pub last_kind: Option<&'static str>,
}

impl<S> Observe<S> for FailureDiagnostics<S::Float>
where
    S: crate::UserState,
    S::Float: FloatCore + Send + Sync,
{
    fn observe(&self, _: &'static str, state: StateView<'_, S>, event: &EngineSignal<S::Float>) {
        let mut inner = self.inner.lock().unwrap();
        match event {
            EngineSignal::Progress(progress) => {
                inner.last_iteration = state.iteration();

                match progress {
                    Progress::Measure(measure) => {
                        inner.last_metric = Some(*measure);
                        inner.last_kind = Some("metric");
                    }

                    Progress::Report { measure, .. } => {
                        inner.last_metric = Some(*measure);
                        inner.last_kind = Some("report");
                    }

                    Progress::Complete => {
                        inner.last_kind = Some("complete");
                    }
                }
            }

            EngineSignal::Termination(_) => {
                // final snapshot already stored
            }

            _ => {}
        }
    }
}
