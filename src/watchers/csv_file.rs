use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use super::ProgressObserver;
use crate::progress::ProgressRow;

pub struct CsvProgressWriter<F> {
    writer: Arc<Mutex<BufWriter>>,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: std::fmt::Display + Send + Sync + 'static> CsvProgressWriter<F> {
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(writer, "iteration,current,best,since_best,termination")?;

        Ok(Self {
            writer: Mutex::new(writer),
            _phantom: Default::default(),
        })
    }
}

impl<F> ProgressObserver<F> for CsvProgressWriter<F>
where
    F: std::fmt::Display + Send + Sync + 'static,
{
    fn observe(&self, row: ProgressRow<F>) {
        let mut w = self.writer.lock().unwrap();

        let _ = writeln!(
            w,
            "{},{},{},{},{:?}",
            row.iteration, row.current, row.best, row.since_best, row.termination
        );
    }
}
