use std::path::{Path, PathBuf};

use crate::engine::EngineStage;
use crate::watchers::{ObservationContext, StateObserver};
use crate::State;

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

    fn should_run(&self, stage: EngineStage) -> bool {
        if self.only_on_wrapup {
            matches!(stage, EngineStage::WrapUp)
        } else {
            true
        }
    }
}

impl<S> StateObserver<S> for PlotObserver
where
    S: crate::UserState,
{
    fn observe(&self, _ident: &'static str, _subject: &State<S>, ctx: &ObservationContext) {
        // Only meaningful at end-of-run (or full tracing mode)
        if let Err(e) = plot_csv(&self.csv_path, &self.output_dir) {
            // Observers should not fail the engine.
            // In a more advanced setup, you might route this to a logging observer.
            eprintln!("PlotObserver failed: {e}");
        }
    }

    fn should_observe(&self, stage: EngineStage) -> bool {
        stage == EngineStage::WrapUp
    }
}
