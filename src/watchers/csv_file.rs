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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct Row<F> {
    pub(super) iteration: usize,
    pub(super) absolute: Option<F>,
    pub(super) relative: Option<F>,
    pub(super) value: Option<F>,
    pub(super) tag: String,
}

pub(super) fn load_csv<F: DeserializeOwned>(
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

fn parse_opt(s: Option<&str>) -> Option<f64> {
    match s {
        Some(v) if !v.is_empty() => v.parse().ok(),
        _ => None,
    }
}

pub struct CsvProgressWriter<S, W: Write> {
    writer: Arc<Mutex<Writer<W>>>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: std::fmt::Display + Send + Sync + 'static> CsvProgressWriter<S, BufWriter<File>> {
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        let csv_writer = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(writer);

        Ok(Self {
            writer: Arc::new(Mutex::new(csv_writer)),
            _phantom: Default::default(),
        })
    }
}

impl<S, W> Observe<S> for CsvProgressWriter<S, W>
where
    S: UserState + Send + Sync,
    W: Write + Send,
{
    fn observe(&self, name: &'static str, state: StateView<'_, S>, event: &EngineSignal<S::Float>) {
        if let EngineSignal::Progress(progress) = event {
            let iteration = state.iteration();
            let row = match progress {
                Progress::Metric { value } => Row {
                    iteration,
                    absolute: None,
                    relative: None,
                    value: Some(value),
                    tag: event.as_tag().to_string(),
                },
                Progress::ErrorEstimate { absolute, relative } => Row {
                    iteration,
                    absolute: Some(absolute),
                    relative: Some(relative),
                    value: None,
                    tag: event.as_tag().to_string(),
                },
                _ => return,
            };
            let mut w = self.writer.lock().unwrap();

            if w.serialize(row).is_err() {
                eprintln!("failed to serialize csv file row");
            }
        }
    }
}
