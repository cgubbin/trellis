/// Numerical signals emitted by a solver during execution.
///
/// `Progress` represents *algorithm-level observations*, not engine control flow.
/// These signals are produced by a [`crate::Procedure`] and consumed by:
/// - convergence tracking
/// - policy evaluation
/// - monitoring / logging systems
#[derive(Clone, Debug)]
pub enum Progress<F> {
    /// Primary metric (loss, objective function, error estimate etc)
    Measure(F),
    Report {
        measure: F,
        diagnostics: ProgressDiagnostics<F>,
    },
    /// Signals that the algorithm is complete
    ///
    /// This is a semantic signal from the algorithm, distinct from policy decisions implemented
    /// directly by the engine
    Complete,
}

#[derive(Clone, Debug, Default)]
pub struct ProgressDiagnostics<F> {
    pub absolute_error: Option<F>,
    pub relative_error: Option<F>,
    pub gradient_norm: Option<F>,
    pub step_size: Option<F>,
}
