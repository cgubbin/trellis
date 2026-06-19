use crate::progress::{Progress, ProgressDiagnostics};
use crate::TrellisFloat;

/// The user-defined state must implement this trait to be used as part of the trellis calculation
/// loop
///
/// All other state methods are auto-implemented on a type wrapping the user-defined state.
///
/// TODO: At the moment we have a clone bound here to enable checkpointing. This is not ideal
pub trait UserState: Clone + Default {
    type Float: TrellisFloat;

    /// Reports progress AFTER update
    fn progress(&self) -> Progress<Self::Float>;
}

pub trait HasParams: UserState {
    type Param;

    fn get_param(&self) -> Option<&Self::Param>;
}

pub trait Snapshotable: UserState {
    type Snapshot: Clone + Send + Sync + 'static;

    fn snapshot(&self) -> Self::Snapshot;
    fn restore(snapshot: Self::Snapshot) -> Self;
}

pub trait DiagnosticState: UserState {
    fn diagnostics(&self) -> ProgressDiagnostics<Self::Float>;
}
