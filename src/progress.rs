#[derive(Clone, Debug, PartialEq)]
pub enum Progress<F> {
    Metric { value: F },
    ErrorEstimate { absolute: F, relative: F },
    Complete,
}

#[derive(Clone, Debug)]
pub struct ProgressReport<F> {
    pub measure: Progress<F>,
    pub diagnostics: ProgressDiagnostics<F>,
}

#[derive(Clone, Debug)]
pub struct ProgressDiagnostics<F> {
    pub gradient_norm: Option<F>,
    pub step_size: Option<F>,
    pub acceptance_rate: Option<F>,
}

#[derive(Debug, Clone)]
pub struct ProgressRow<F> {
    pub iteration: usize,
    pub current: F,
    pub best: F,
    pub since_best: usize,
    pub termination: Option<crate::Termination>,
}

impl<S> From<&crate::State<S>> for ProgressRow<S::Float>
where
    S: crate::UserState,
{
    fn from(state: &crate::State<S>) -> Self {
        Self {
            iteration: state.runtime.iteration(),
            current: state.convergence.current(),
            best: state.convergence.best(),
            since_best: state.iterations_since_best(),
            termination: state.runtime.termination(),
        }
    }
}
