use std::path::PathBuf;

use crate::engine::EngineSignal;
use crate::state::StateView;
use crate::watchers::Observe;

mod internal;
use internal::plot_csv;

/// Post-run observer that generates plots from recorded CSV output.
///
/// This observer does not participate in runtime analysis; it only
/// reacts to completion of a run and performs offline visualization.
pub struct PlotObserver {
    csv_path: PathBuf,
    output_dir: PathBuf,
}

impl PlotObserver {
    /// Create a new plot observer.
    pub fn new(csv_path: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            csv_path: csv_path.into(),
            output_dir: output_dir.into(),
        }
    }
}

impl<S> Observe<S> for PlotObserver
where
    S: crate::UserState,
{
    fn observe(&self, _: &'static str, _: StateView<'_, S>, event: &EngineSignal<S::Float>) {
        if let EngineSignal::Termination(_) = event {
            if let Err(e) = plot_csv(&self.csv_path, &self.output_dir) {
                eprintln!("PlotObserver failed: {e}");
            }
        }
    }
}
