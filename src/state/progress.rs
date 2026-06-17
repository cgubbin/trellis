#[derive(Clone, Debug)]
pub enum Progress<F> {
    Metric { value: F },
    ErrorEstimate { absolute: F, relative: F },
}
