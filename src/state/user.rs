use crate::progress::Progress;
use crate::TrellisFloat;

/// The user-defined state must implement this trait to be used as part of the trellis calculation
/// loop
///
/// All other state methods are auto-implemented on a type wrapping the user-defined state.
pub trait UserState {
    type Float: TrellisFloat;
    // type Param;

    fn is_initialised(&self) -> bool {
        true
    }

    // fn get_param(&self) -> Option<&Self::Param>;

    fn progress(&self) -> Progress<Self::Float>;
}

pub trait Snapshotable {
    type Snapshot: Clone + Send + Sync + 'static;

    fn snapshot(&self) -> Self::Snapshot;
}

pub trait StateRestorer<S: Snapshotable> {
    fn restore(snapshot: S::Snapshot) -> S;
}
