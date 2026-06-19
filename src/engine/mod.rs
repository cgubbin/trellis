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
//! Checkpoints are optional and handled via [`CheckpointStore`].
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
mod policy;
mod result;
mod termination;

use crate::progress::Progress;
pub use builder::GenerateBuilder;
pub use cancellation::CancellationGuard;
pub use checkpoint::{Checkpoint, CheckpointStore};
use context::EngineContext;
pub(crate) use event::{EngineAction, EngineSignal, EventBatch};
use policy::{EnginePolicy, PolicyStack};
pub use result::{EngineFailure, EngineOutput, EngineResult};
pub use termination::Termination;

use num_traits::float::FloatCore;
use std::time::Instant;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    state::{State, StateView},
    Problem, Procedure,
};
use crate::{watchers::Observers, Output, UserState};

pub type Error = Box<dyn std::error::Error>;

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
pub struct Engine<P, Q, C>
where
    P: Procedure,
    P::State: UserState,
    Q: EnginePolicy<<P::State as UserState>::Float>,
{
    /// Procedure to be run
    procedure: P,
    /// The problem to solve
    problem: Problem<P::Problem>,

    policy: Q,
    /// Current state of the run
    state: Option<State<P::State>>,
    /// Should execution be timed
    time: bool,

    start_time: Option<std::time::Instant>,
    /// Receiver
    ///
    /// When a signal is received on this channel the procedure is terminated.
    cancellation: CancellationToken,

    observers: Observers<P::State>,

    checkpoint_store: Option<C>,
}

impl<P, Q, C> Engine<P, Q, C>
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore,
    C: CheckpointStore<P::State>,
    Q: EnginePolicy<<P::State as UserState>::Float>,
{
    pub fn run(mut self) -> EngineResult<P::Output, P::State, P::Error> {
        let mut state = self.state.take().unwrap();

        self.initialise_state(&mut state)
            .map_err(|error| EngineFailure::Procedure {
                error,
                state: state.clone(),
            })?;

        loop {
            let batch = self
                .step_once(&mut state)
                .map_err(|(error, state)| EngineFailure::Procedure { error, state })?;

            let ctx = EngineContext {
                iter: state.runtime.iteration(),
                elapsed: self.start_time().elapsed(),
                cancelled: self.cancellation.is_cancelled(),
                checkpoint_due: false,
                start_time: self.start_time(),
                _marker: Default::default(),
            };

            let action = self.policy.decide(&batch, &ctx);

            match action {
                EngineAction::Continue => continue,
                EngineAction::Stop(reason) => {
                    state.runtime.terminate(reason);
                    self.observe(&state, EngineSignal::Termination(reason));
                    return self.finalise(state);
                }
                EngineAction::EmitCheckpoint(reason) => self.emit_checkpoint(&state, reason),
            }
        }
    }

    fn start_time(&self) -> Instant {
        self.start_time
            .expect("start time should always be set in the initialisation phase")
    }

    #[instrument(name = "initialising runner", fields(ident = P::NAME), skip_all)]
    fn initialise_state(&mut self, state: &mut State<P::State>) -> Result<(), P::Error> {
        self.start_time = Some(Instant::now());
        self.procedure
            .initialise(&mut self.problem, &mut state.user)?;

        self.observe(&state, EngineSignal::Initialised);

        Ok(())
    }

    #[instrument(name = "wrapping up runner", fields(ident = P::NAME), skip_all)]
    fn finalise(
        &mut self,
        mut state: State<P::State>,
    ) -> EngineResult<P::Output, P::State, P::Error> {
        let output = self
            .procedure
            .finalise(&mut self.problem, &mut state.user)
            .map_err(|e| EngineFailure::Procedure {
                error: e,
                state: state.clone(),
            })?;

        let output = Output::new(output, state.clone());

        match state.runtime.termination().unwrap() {
            Termination::Converged => Ok(EngineOutput::Success(output)),
            termination => Ok(EngineOutput::Terminated {
                termination,
                output,
            }),
        }
    }

    fn step_once(
        &mut self,
        state: &mut State<P::State>,
    ) -> Result<EventBatch<<P::State as UserState>::Float>, (P::Error, State<P::State>)> {
        state.runtime.increment_iteration();

        let prev = state.clone();
        self.procedure
            .step(
                &mut self.problem,
                &mut state.user,
                CancellationGuard {
                    token: &self.cancellation,
                },
            )
            .map_err(|e| (e, prev))?;

        let progress = state.user.progress();

        state
            .convergence
            .observe(&progress, state.runtime.iteration());

        self.observe(&state, EngineSignal::Progress(progress.clone()));

        let events = EventBatch::new().add(progress);

        Ok(events)
    }

    fn observe(
        &self,
        state: &State<P::State>,
        stage: EngineSignal<<P::State as UserState>::Float>,
    ) {
        let state_view = StateView::new(state);
        self.observers.dispatch(P::NAME, state_view, &stage);
    }

    fn emit_checkpoint(&self, state: &State<P::State>, _reason: event::CheckpointReason) {
        if let Some(checkpoint_store) = self.checkpoint_store.as_ref() {
            let checkpoint = Checkpoint::new(state);
            if let Err(e) = checkpoint_store.save(&checkpoint) {
                tracing::warn!(error = ?e, "checkpoint save failed");
            }
            self.observe(&state, EngineSignal::CheckpointSaved);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{
        engine::checkpoint::InMemoryCheckpointStore,
        engine::policy::{CheckpointPolicy, MaxIterationPolicy, TargetValuePolicy},
        progress::Progress,
        Problem,
    };

    struct Dummy;

    #[derive(Clone, Default, Debug)]
    struct DummyState {
        iter: usize,
        value: f64,
    }

    #[derive(thiserror::Error, Debug)]
    enum DummyError {}

    impl UserState for DummyState {
        type Float = f64;

        fn progress(&self) -> Progress<Self::Float> {
            let rep = Progress::Metric { value: self.value };

            dbg!(&rep);
            rep
        }
    }

    impl Procedure for Dummy {
        type Error = DummyError;

        type State = DummyState;
        type Problem = ();
        type Output = ();

        const NAME: &'static str = "Dummy Procedure";

        fn initialise(
            &mut self,
            _: &mut Problem<()>,
            _: &mut DummyState,
        ) -> Result<(), DummyError> {
            Ok(())
        }

        fn step(
            &mut self,
            _: &mut Problem<()>,
            state: &mut DummyState,
            _: CancellationGuard,
        ) -> Result<(), DummyError> {
            state.iter += 1;
            state.value -= 1.0;
            Ok(())
        }

        fn finalise(&mut self, _: &mut Problem<()>, _: &DummyState) -> Result<(), DummyError> {
            Ok(())
        }

        fn is_finished(&self, state: &Self::State) -> bool {
            false
        }
    }

    #[test]
    fn engine_runs_and_stops_on_policy() {
        let engine = Dummy
            .build_for(())
            .and_policy(TargetValuePolicy::new(0.0))
            .finalise();

        let result = engine.run();

        assert!(result.is_ok());
    }

    #[test]
    fn engine_propagates_termination_reason() {
        let engine = Dummy
            .build_for(())
            .and_policy(MaxIterationPolicy::new(3))
            .finalise();

        let result = engine.run().unwrap();

        match result {
            EngineOutput::Terminated { termination, .. } => {
                assert_eq!(termination, Termination::ExceededMaxIterations);
            }
            _ => panic!("Expected termination"),
        }
    }

    use crate::watchers::{Frequency, Observe};
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    struct Spy {
        pub called: AtomicUsize,
    }

    impl<S: UserState> Observe<S> for Spy {
        fn observe(
            &self,
            _id: &'static str,
            _state: StateView<S>,
            _event: &EngineSignal<S::Float>,
        ) {
            self.called.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn observers_are_called_during_execution() {
        let spy = Arc::new(Spy {
            called: AtomicUsize::new(0),
        });

        let engine = Dummy
            .build_for(())
            .attach_observer(spy.clone(), Frequency::Always)
            .and_policy(MaxIterationPolicy::new(10))
            .finalise();

        let _ = engine.run();

        assert!(spy.called.load(Ordering::SeqCst) > 0);
    }
    #[test]
    fn checkpoint_is_triggered_by_policy() {
        let store = InMemoryCheckpointStore::new();

        let engine = Dummy
            .build_for(())
            .with_checkpoint_store(store.clone())
            .and_policy(CheckpointPolicy::every(5))
            .and_policy(MaxIterationPolicy::new(10))
            .finalise();

        let _ = engine.run();

        assert!(store.saved_count() > 0);
    }

    // #[test]
    // fn full_convergence_pipeline() {
    //     let engine = Dummy
    //         .build_for(())
    //         .and_policy(TargetValuePolicy::new(0.01))
    //         .attach_observer(Tracer::new(Level::INFO), Frequency::Always)
    //         .finalise();

    //     let result = engine.run().unwrap();

    //     assert!(matches!(result, EngineOutput::Success(_)));
    // }
}

//     #[test]
//     fn stagnation_terminates_engine() {
//         let engine = Dummy
//             .build_for(())
//             .and_policy(StagnationPolicy::new(5))
//             .finalise();

//         let result = engine.run().unwrap();

//         match result {
//             EngineOutput::Terminated { termination, .. } => {
//                 assert_eq!(termination, Termination::Stagnated);
//             }
//             _ => panic!(),
//         }
//     }

//     #[test]
//     fn cancellation_stops_engine() {
//         let token = CancellationToken::new();
//         token.cancel();

//         let engine = Dummy
//             .build_for(())
//             .cancellation_token(token)
//             .and_policy(CancellationPolicy)
//             .finalise();

//         let result = engine.run().unwrap();

//         assert!(matches!(
//             result,
//             EngineOutput::Terminated {
//                 termination: Termination::Cancelled,
//                 ..
//             }
//         ));
//     }
// }
