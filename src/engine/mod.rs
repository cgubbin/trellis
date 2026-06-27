//! # Engine
//!
//! This module implements the core execution loop for running iterative numerical or
//! procedural computations.
//!
//! The engine is designed around four interacting subsystems:
//!
//! - **Procedure**: user-defined computation (initialise → step → finalise)
//! - **State**: mutable runtime state of the system
//! - **Policy system**: decides when to continue, stop, or checkpoint
//! - **Observers**: side-effect systems (logging, CSV export, plotting, tracing)
//!
//! ## Execution model
//!
//! The engine runs an iterative loop:
//!
//! 1. Initialise state via `Procedure::initialise`
//! 2. Repeatedly call `Procedure::step`
//! 3. Extract `Progress` from user state
//! 4. Feed progress into:
//!    - convergence tracking
//!    - observers
//!    - policy system
//! 5. Execute policy decision:
//!    - Continue
//!    - Stop
//!    - Emit checkpoint
//!
//! ## Event-driven design
//!
//! The engine communicates internally using lightweight event structures:
//!
//! - [`Progress`] — numeric or semantic convergence signal
//! - [`EventBatch`] — per-iteration aggregation of events
//! - [`EngineAction`] — decision output of the policy system
//!
//! ## Checkpointing
//!
//! Checkpoints are optional and handled via [`CheckpointBackend`].
//! They are triggered by policy decisions, not directly by the procedure.
//!
//! ## Observers
//!
//! Observers receive immutable views of state via [`StateView`] and are decoupled
//! from execution logic. They are used for:
//!
//! - logging (`Tracer`)
//! - persistence (CSV / JSON)
//! - plotting / visualization
//!
//! Observers do not affect control flow.
//!
//! ## Policy system
//!
//! Policies consume event batches + context and return an [`EngineAction`]:
//!
//! - Continue execution
//! - Stop execution (with termination reason)
//! - Request checkpoint
//!
//! Policies are composable via [`PolicyStack`].
mod builder;
mod cancellation;
mod checkpoint;
mod context;
mod event;
mod extensions;
mod policy;
mod result;
mod termination;

pub use policy::{
    AbsoluteTolerancePolicy, CheckpointPolicy, EnginePolicy, MaxIterationPolicy, NoProgressPolicy,
    RelativeTolerancePolicy, StagnationPolicy, TargetValuePolicy, TimeoutPolicy,
};

pub use builder::{GenerateBuilder, GenerateBuilderFallible};
pub use cancellation::CancellationGuard;
use context::EngineContext;
pub(crate) use event::{EngineAction, EngineSignal, EventBatch};
use extensions::Extensions;

pub use result::{EngineFailure, EngineResult, EngineResultWithSnapshot};
use result::{InternalEngineFailure, InternalEngineResult};
pub use termination::Termination;

pub use checkpoint::InMemoryCheckpointStore;

#[cfg(feature = "writing")]
pub use checkpoint::JsonCheckpointStore;

use num_traits::float::FloatCore;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    result::EngineOutput,
    state::{Snapshotable, State, StateView},
    FallibleProcedure,
};
use crate::{watchers::Observers, UserState};

/// Core execution engine for iterative procedures.
///
/// The engine owns:
/// - a [`Procedure`] implementation
/// - a mutable [`State`]
/// - a policy stack controlling termination/checkpointing
/// - optional observers for side effects
/// - optional checkpoint storage
///
/// The engine is generic over:
/// - `P`: the computation procedure
/// - `Q`: policy implementation (usually [`PolicyStack`])
/// - `C`: checkpoint storage backend
pub struct Engine<Proc, P, Q>
where
    Proc: FallibleProcedure<P>,
    Proc::State: UserState,
    Q: EnginePolicy<<Proc::State as UserState>::Float>,
{
    /// Procedure to be run
    procedure: Proc,
    /// The problem to solve
    problem: P,

    policy: Q,
    /// Current state of the run
    state: State<Proc::State>,
    /// Should execution be timed
    time: bool,

    start_time: Option<std::time::Instant>,
    /// Receiver
    ///
    /// When a signal is received on this channel the procedure is terminated.
    cancellation: CancellationToken,

    observers: Observers<Proc::State>,

    extensions: Extensions<Proc::State>,
}

impl<Proc, P, Q> Engine<Proc, P, Q>
where
    Proc: FallibleProcedure<P>,
    Proc::State: UserState,
    <Proc::State as UserState>::Float: FloatCore,
    Q: EnginePolicy<<Proc::State as UserState>::Float>,
{
    pub fn run_with_snapshot(
        mut self,
    ) -> EngineResultWithSnapshot<Proc::Output, Proc::State, Proc::Error>
    where
        Proc::State: Snapshotable,
    {
        let result = self._run();

        let snapshot = self.state.user.snapshot();

        result
            .map(|output| output.with_snapshot(snapshot))
            .map_err(|internal| EngineFailure::from_internal(internal, self.state))
    }

    pub fn run(mut self) -> EngineResult<Proc::Output, Proc::State, Proc::Error> {
        self._run()
            .map_err(|internal| EngineFailure::from_internal(internal, self.state))
    }

    fn _run(&mut self) -> InternalEngineResult<Proc::Output, Proc::State, Proc::Error> {
        self.initialise_state()?;

        loop {
            let result = self.policy_step()?;

            match result {
                EngineAction::Continue => continue,
                EngineAction::Stop(reason) => {
                    self.emit_event(EngineSignal::Termination(reason));
                    return self.finalise(reason);
                }
                EngineAction::EmitCheckpoint(reason) => {
                    self.emit_event(EngineSignal::CheckpointRequested(reason));
                }
            }
        }
    }

    fn policy_step(&mut self) -> Result<EngineAction, InternalEngineFailure<Proc::Error>> {
        let batch = self.step_once()?;

        let ctx = EngineContext {
            iter: self.state.runtime.iteration(),
            elapsed: self.start_time().elapsed(),
            cancelled: self.cancellation.is_cancelled(),
            checkpoint_due: false,
            start_time: self.start_time(),
            _marker: Default::default(),
        };

        let action = self.policy.decide(&batch, &ctx);
        Ok(action)
    }

    fn start_time(&self) -> Instant {
        self.start_time
            .expect("start time should always be set in the initialisation phase")
    }

    #[instrument(name = "initialising runner", fields(ident = Proc::NAME), skip_all)]
    fn initialise_state(&mut self) -> Result<(), InternalEngineFailure<Proc::Error>> {
        self.start_time = Some(Instant::now());
        self.state
            .runtime
            .record_duration(Instant::now() - self.start_time.unwrap());

        self.procedure
            .initialise_fallible(&mut self.problem, &mut self.state.user)
            .map_err(InternalEngineFailure::new)?;

        self.emit_event(EngineSignal::Initialised);

        Ok(())
    }

    #[instrument(name = "wrapping up runner", fields(ident = Proc::NAME), skip_all)]
    fn finalise(
        &mut self,
        reason: Termination,
    ) -> InternalEngineResult<Proc::Output, Proc::State, Proc::Error> {
        match self
            .procedure
            .finalise_fallible(&mut self.problem, &self.state.user)
        {
            Err(e) => Err(InternalEngineFailure::new(e)),
            Ok(result) => {
                self.state
                    .runtime
                    .record_duration(Instant::now() - self.start_time.unwrap());

                Ok(EngineOutput::new(
                    result,
                    StateView::new(&self.state),
                    reason,
                ))
            }
        }
    }

    fn step_once(
        &mut self,
    ) -> Result<EventBatch<<Proc::State as UserState>::Float>, InternalEngineFailure<Proc::Error>>
    {
        self.state.runtime.increment_iteration();
        self.state
            .runtime
            .record_duration(Instant::now() - self.start_time.unwrap());

        self.procedure
            .step_fallible(
                &mut self.problem,
                &mut self.state.user,
                CancellationGuard {
                    token: &self.cancellation,
                },
            )
            .map_err(InternalEngineFailure::new)?;

        let progress = self.state.user.progress();

        self.state
            .convergence
            .observe(&progress, self.state.runtime.iteration());

        self.emit_event(EngineSignal::Progress(progress.clone()));

        let events = EventBatch::new().add(progress);

        Ok(events)
    }

    fn emit_event(&mut self, signal: EngineSignal<<Proc::State as UserState>::Float>) {
        let state_view = StateView::new(&self.state);

        self.extensions.dispatch(state_view, &signal);

        self.observers.dispatch(Proc::NAME, state_view, &signal);
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     use crate::{
//         engine::checkpoint::InMemoryCheckpointStore,
//         engine::policy::{CheckpointPolicy, MaxIterationPolicy, TargetValuePolicy},
//         progress::Progress,
//         Problem,
//     };

//     struct Dummy;

//     #[derive(Clone, Default, Debug)]
//     struct DummyState {
//         iter: usize,
//         value: f64,
//     }

//     #[derive(thiserror::Error, Debug)]
//     enum DummyError {}

//     impl UserState for DummyState {
//         type Float = f64;

//         fn progress(&self) -> Progress<Self::Float> {
//             let rep = Progress::Measure(self.value);

//             dbg!(&rep);
//             rep
//         }
//     }

//     impl Snapshotable for DummyState {
//         type Snapshot = Self;

//         fn snapshot(&self) -> Self::Snapshot {
//             self.clone()
//         }
//     }

//     impl Procedure for Dummy {
//         type Error = DummyError;

//         type State = DummyState;
//         type Problem = ();
//         type Output = ();

//         const NAME: &'static str = "Dummy Procedure";

//         fn initialise(
//             &self,
//             _: &Problem<()>,
//             _: &mut DummyState,
//         ) -> Result<(), DummyError> {
//             Ok(())
//         }

//         fn step(
//             &mut self,
//             _: &mut Problem<()>,
//             state: &mut DummyState,
//             _: CancellationGuard,
//         ) -> Result<(), DummyError> {
//             state.iter += 1;
//             state.value -= 1.0;
//             Ok(())
//         }

//         fn finalise(&mut self, _: &mut Problem<()>, _: &DummyState) -> Result<(), DummyError> {
//             Ok(())
//         }

//         fn is_finished(&self, state: &Self::State) -> bool {
//             false
//         }
//     }

//     #[test]
//     fn engine_runs_and_stops_on_policy() {
//         let engine = Dummy
//             .build_for(())
//             .and_policy(TargetValuePolicy::new(0.0))
//             .with_initial_state(DummyState::default())
//             .finalise();

//         let result = engine.run();

//         assert!(result.is_ok());
//     }

//     #[test]
//     fn engine_propagates_termination_reason() {
//         let engine = Dummy
//             .build_for(())
//             .and_policy(MaxIterationPolicy::new(3))
//             .with_initial_state(DummyState::default())
//             .finalise();

//         let result = engine.run().unwrap();

//         assert_eq!(result.termination, Termination::ExceededMaxIterations);
//     }

//     use crate::watchers::{Frequency, Observe};
//     use std::sync::atomic::AtomicUsize;
//     use std::sync::atomic::Ordering;
//     use std::sync::Arc;

//     struct Spy {
//         pub called: AtomicUsize,
//     }

//     impl<S: UserState> Observe<S> for Spy {
//         fn observe(
//             &self,
//             _id: &'static str,
//             _state: StateView<S>,
//             _event: &EngineSignal<S::Float>,
//         ) {
//             self.called.fetch_add(1, Ordering::SeqCst);
//         }
//     }

//     #[test]
//     fn observers_are_called_during_execution() {
//         let spy = Arc::new(Spy {
//             called: AtomicUsize::new(0),
//         });

//         let engine = Dummy
//             .build_for(())
//             .attach_observer(spy.clone(), Frequency::Always)
//             .and_policy(MaxIterationPolicy::new(10))
//             .with_initial_state(DummyState::default())
//             .finalise();

//         let _ = engine.run();

//         assert!(spy.called.load(Ordering::SeqCst) > 0);
//     }
//     #[test]
//     fn checkpoint_is_triggered_by_policy() {
//         let store = InMemoryCheckpointStore::new();

//         let engine = Dummy
//             .build_for(())
//             .with_checkpoint_backend(store.clone())
//             .and_policy(CheckpointPolicy::every(5))
//             .and_policy(MaxIterationPolicy::new(10))
//             .with_initial_state(DummyState::default())
//             .finalise();

//         let _ = engine.run();

//         assert!(store.saved_count() > 0);
//     }

//     #[test]
//     fn full_convergence_pipeline() {
//         let engine = Dummy
//             .build_for(())
//             .and_policy(TargetValuePolicy::new(0.01))
//             .attach_observer(
//                 crate::watchers::Tracer::new(tracing::Level::INFO),
//                 Frequency::Always,
//             )
//             .with_initial_state(DummyState::default())
//             .finalise();

//         let result = engine.run().unwrap();

//         assert_eq!(result.termination, Termination::Converged);
//     }
// }
