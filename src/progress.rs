#[derive(Clone, Debug, PartialEq)]
pub enum Progress<F> {
    Metric { value: F },
    ErrorEstimate { absolute: F, relative: F },
    Complete,
}

#[derive(Clone, Debug)]
pub struct ProgressDiagnostics<F> {
    pub gradient_norm: Option<F>,
    pub step_size: Option<F>,
    pub acceptance_rate: Option<F>,
}
