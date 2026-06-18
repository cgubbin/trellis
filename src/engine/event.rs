use crate::progress::Progress;
use crate::Termination;

#[derive(PartialEq)]
pub enum RawEvent<F> {
    Progress(Progress<F>),

    Iteration { iter: usize },

    ErrorUpdated { value: F },
}

pub enum EngineEvent<F> {
    Progress(Progress<F>),

    BestImproved,

    NoImprovementWindow { window: usize },

    Converged,

    Stagnated,

    TerminationRequested(Termination),

    CheckpointRequested,

    Pass,
}
