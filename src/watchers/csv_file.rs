use csv::Reader;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use super::ProgressObserver;
use crate::engine::EngineStage;
use crate::progress::ProgressRow;

#[derive(Debug, Clone)]
pub(super) struct Row {
    pub(super) iteration: usize,
    pub(super) absolute: Option<f64>,
    pub(super) relative: Option<f64>,
    pub(super) value: Option<f64>,
    pub(super) tag: String,
}

pub(super) fn load_csv(path: impl AsRef<std::path::Path>) -> Result<Vec<Row>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(path)?;

    let mut rows = Vec::new();

    for result in rdr.records() {
        let r = result?;

        let row = Row {
            iteration: r.get(0).unwrap().parse()?,
            absolute: parse_opt(r.get(1)),
            relative: parse_opt(r.get(2)),
            value: parse_opt(r.get(3)),
            tag: r.get(4).unwrap_or("").to_string(),
        };

        rows.push(row);
    }

    Ok(rows)
}

fn parse_opt(s: Option<&str>) -> Option<f64> {
    match s {
        Some(v) if !v.is_empty() => v.parse().ok(),
        _ => None,
    }
}

pub struct CsvProgressWriter<F, W: Write> {
    writer: Arc<Mutex<BufWriter<W>>>,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: std::fmt::Display + Send + Sync + 'static> CsvProgressWriter<F, File> {
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "iteration,current,best,since_best,termination")?;

        Ok(Self {
            writer: Arc::new(Mutex::new(writer)),
            _phantom: Default::default(),
        })
    }
}

impl<F, W> ProgressObserver<F> for CsvProgressWriter<F, W>
where
    F: std::fmt::Display + Send + Sync + 'static,
    W: Write + Send,
{
    fn observe(&self, row: ProgressRow<F>) {
        let mut w = self.writer.lock().unwrap();

        let _ = writeln!(
            w,
            "{},{},{},{},{:?}",
            row.iteration, row.current, row.best, row.since_best, row.termination
        );
    }

    fn should_observe(&self, stage: EngineStage) -> bool {
        stage == EngineStage::Iteration
    }
}
