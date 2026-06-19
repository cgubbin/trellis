mod convergence;
mod runtime;
mod view;

use crate::progress::ProgressReport;
use crate::TrellisFloat;
use convergence::ConvergenceState;
use runtime::RuntimeState;

pub(crate) use view::StateView;

use num_traits::float::FloatCore;
use serde::{Deserialize, Serialize};

/// The user-defined state must implement this trait to be used as part of the trellis calculation
/// loop
///
/// All other state methods are auto-implemented on a type wrapping the user-defined state.
///
/// TODO: At the moment we have a clone bound here to enable checkpointing. This is not ideal
/// because the user state could be large. In future we may want to introduce a new trait:
/// pub trait Checkpointable {
///    type Checkpoint;
///
///    fn checkpoint(&self) -> Self::Checkpoint;
///    }
/// Which could be implemented or not for a given state.
pub trait UserState: Clone + Default {
    type Float: TrellisFloat;
    type Param;
    type Snapshot: Clone;

    // Returns the current parameter value, if one is assigned
    fn get_param(&self) -> Option<&Self::Param>;

    /// Reports progress AFTER update
    fn progress(&self) -> ProgressReport<Self::Float>;

    fn snapshot(&self) -> Self::Snapshot;
}

/// The state of the [`trellis`] solver
///
/// This contains generic fields common to all solvers, as well as a user-defined state
/// `S` which contains application specific fields.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct State<S: UserState> {
    /// The user component of the state implements the application specific code
    pub(crate) user: S,

    pub(crate) runtime: RuntimeState,

    pub(crate) convergence: ConvergenceState<S::Float>,
}

impl<S> State<S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    /// Create a new instance of the iteration state
    pub(crate) fn new() -> Self {
        Self {
            user: S::default(),
            runtime: RuntimeState::new(),
            convergence: ConvergenceState::new(),
        }
    }

    /// Returns the number of iterations since the best result was observed
    pub(crate) fn iterations_since_best(&self) -> usize {
        self.convergence
            .iterations_since_best(self.runtime.iteration())
    }

    pub fn record(&mut self, value: S::Float) -> bool {
        self.convergence.record(value, self.runtime.iteration())
    }
}
