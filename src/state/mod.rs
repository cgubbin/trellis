mod convergence;
mod runtime;
mod user;
mod view;

use crate::TrellisFloat;
use convergence::ConvergenceState;
use runtime::RuntimeState;

pub use user::{Snapshotable, UserState};
pub(crate) use view::StateView;

use num_traits::float::FloatCore;
use serde::{Deserialize, Serialize};

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
