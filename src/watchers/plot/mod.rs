use std::path::PathBuf;

use crate::engine::EngineSignal;
use crate::state::StateView;
use crate::watchers::Observe;

/// Reuses your existing plotting pipeline
mod plot;
use plot::plot_csv;

/// State-level observer that produces plots at the end of a run.
///
/// This assumes a ProgressCsvWriter (or equivalent) has already written the CSV file.
pub struct PlotObserver {
    csv_path: PathBuf,
    output_dir: PathBuf,
    only_on_wrapup: bool,
}

impl PlotObserver {
    pub fn new(csv_path: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            csv_path: csv_path.into(),
            output_dir: output_dir.into(),
            only_on_wrapup: true,
        }
    }

    pub fn run_on_all_stages(mut self) -> Self {
        self.only_on_wrapup = false;
        self
    }

    fn should_run<F>(&self, event: EngineSignal<F>) -> bool {
        if self.only_on_wrapup {
            matches!(event, EngineSignal::Termination(_))
        } else {
            true
        }
    }
}

impl<S> Observe<S> for PlotObserver
where
    S: crate::UserState,
{
    fn observe(
        &self,
        _ident: &'static str,
        _state: StateView<'_, S>,
        event: &EngineSignal<S::Float>,
    ) {
        match event {
            EngineSignal::Termination(..) => {
                // Only meaningful at end-of-run (or full tracing mode)
                if let Err(e) = plot_csv(&self.csv_path, &self.output_dir) {
                    // Observers should not fail the engine.
                    // In a more advanced setup, you might route this to a logging observer.
                    eprintln!("PlotObserver failed: {e}");
                }
            }
            _ => {}
        }
    }
}
