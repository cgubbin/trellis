use csv::{Reader, Writer};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use super::Observe;
use crate::engine::EngineSignal;
use crate::progress::Progress;
use crate::state::{StateView, UserState};

/// A single row in the CSV export of engine progress.
///
/// This represents a *time-series projection* of the solver state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row<F> {
    pub iteration: usize,

    /// Kind of progress event (metric / report / complete)
    pub kind: String,

    /// Primary scalar measure (if available)
    pub metric: Option<F>,

    /// Absolute error estimate (if available)
    pub absolute: Option<F>,

    /// Relative error estimate (if available)
    pub relative: Option<F>,
}

/// Load previously recorded CSV rows.
pub fn load_csv<F: DeserializeOwned>(
    path: impl AsRef<std::path::Path>,
) -> Result<Vec<Row<F>>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;
    let mut rows = Vec::new();

    for result in rdr.deserialize() {
        let record: Row<F> = result?;
        rows.push(record);
    }

    Ok(rows)
}

/// CSV observer that records progress events as structured rows.
///
/// This is intended for offline analysis, plotting, and diagnostics.
pub struct CsvProgressWriter<S, W: Write> {
    writer: Arc<Mutex<Writer<W>>>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: Send + Sync + 'static, W: Write + Send> CsvProgressWriter<S, W> {
    /// Create a new CSV writer from any writable sink.
    pub fn new(writer: W) -> Self {
        let csv_writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        Self {
            writer: Arc::new(Mutex::new(csv_writer)),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S: Send + Sync + 'static> CsvProgressWriter<S, BufWriter<File>> {
    /// Convenience constructor for file-based output.
    pub fn new_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        Ok(Self::new(writer))
    }
}

impl<S, W> Observe<S> for CsvProgressWriter<S, W>
where
    S: UserState + Send + Sync,
    S::Float: Copy + Serialize,
    W: Write + Send,
{
    fn observe(&self, _: &'static str, state: StateView<'_, S>, event: &EngineSignal<S::Float>) {
        let row = match event {
            EngineSignal::Progress(progress) => {
                let iteration = state.iteration();

                match progress {
                    Progress::Measure(value) => Row {
                        iteration,
                        kind: "metric".to_string(),
                        metric: Some(*value),
                        absolute: None,
                        relative: None,
                    },

                    Progress::Report {
                        measure,
                        diagnostics,
                    } => Row {
                        iteration,
                        kind: "report".to_string(),
                        metric: Some(*measure),
                        absolute: diagnostics.absolute_error,
                        relative: diagnostics.relative_error,
                    },

                    Progress::Complete => Row {
                        iteration,
                        kind: "complete".to_string(),
                        metric: None,
                        absolute: None,
                        relative: None,
                    },
                }
            }

            _ => return,
        };

        let mut writer = self.writer.lock().unwrap();
        let _ = writer.serialize(row);
    }
}
