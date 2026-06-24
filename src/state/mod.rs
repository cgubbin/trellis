/// Core solver state representation.
///
/// This module defines the runtime state used by the engine during execution.
///
/// The state is split into three components:
///
/// - `user`: application-defined state implementing [`UserState`]
/// - `runtime`: engine execution metadata (iteration count, timing)
/// - `convergence`: tracking of numerical convergence behaviour
///
/// ## Design principles
///
/// - The engine owns a single mutable `State<S>` during execution
/// - External code interacts with state via [`StateView`], not direct access
/// - Persistence is handled via [`Snapshotable`] + checkpointing systems
///
/// ## Separation of concerns
///
/// - `UserState` defines problem-specific behaviour
/// - `RuntimeState` tracks execution lifecycle
/// - `ConvergenceState` tracks numerical progress
///
/// These components are intentionally isolated to avoid coupling
/// solver logic with infrastructure concerns.
///
/// ## Notes
///
/// - `State` is not exposed mutably outside the engine
/// - Access is mediated via `StateView<'_>`
/// - Persistence uses snapshot types derived from `Snapshotable`
mod convergence;
mod runtime;
mod user;
mod view;

pub use user::{Snapshotable, StateRestorer, UserState};

pub(crate) use convergence::ConvergenceState;
pub(crate) use runtime::RuntimeState;

pub(crate) use view::StateView;

use num_traits::float::FloatCore;

/// Internal execution state of the solver.
///
/// This struct is owned exclusively by the Engine during execution
/// and is not intended to be modified directly by users.
///
/// Access to state is provided through StateView.
///
/// # Fields
/// - `user`: user-defined state implementing UserState
/// - `runtime`: execution metadata (iteration count, duration)
/// - `convergence`: convergence tracking for policies
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "S: serde::Serialize, S::Float: serde::Serialize",
        deserialize = "S: serde::Deserialize<'de>, S::Float: serde::Deserialize<'de>"
    ))
)]
pub struct State<S: UserState> {
    pub(crate) runtime: RuntimeState,

    pub(crate) convergence: ConvergenceState<S::Float>,
    /// The user component of the state implements the application specific code
    pub user: S,
}

impl<S> State<S>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    /// Creates a fresh solver state.
    ///
    /// Initializes:
    /// - runtime counters at zero
    /// - convergence tracking at initial values
    pub(crate) fn new(user: S) -> Self {
        Self {
            user,
            runtime: RuntimeState::new(),
            convergence: ConvergenceState::new(),
        }
    }

    // TODO: More elegant to not expose these methods, or to return a state at all and construct
    // directly on the error type
    pub fn run_summary(&self) -> crate::RunSummary<<S as UserState>::Float> {
        let view = StateView::new(self);

        crate::RunSummary::new(view)
    }

    pub fn into_user(self) -> S {
        self.user
    }
}
