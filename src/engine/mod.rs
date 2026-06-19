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
pub(crate) use event::{EngineAction, EngineEvent, EventBatch};
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

/// General purpose calculation engine
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
            let events = self
                .step_once(&mut state)
                .map_err(|(error, state)| EngineFailure::Procedure { error, state })?;

            let ctx = EngineContext {
                iter: state.runtime.iteration(),
                elapsed: Instant::now() - self.start_time(),
                cancelled: self.cancellation.is_cancelled(),
                checkpoint_due: false,
                start_time: self.start_time(),
                _marker: Default::default(),
            };

            let batch = EventBatch {
                events: events.clone(),
            };

            let action = self.policy.decide(&batch, &ctx);

            match action {
                EngineAction::Continue => continue,
                EngineAction::Stop(reason) => {
                    state.runtime.terminate(reason);
                    self.observe(&state, EngineEvent::Termination(reason));
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

        self.observe(&state, EngineEvent::Initialised);

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
    ) -> Result<Vec<Progress<<P::State as UserState>::Float>>, (P::Error, State<P::State>)> {
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
            .observe(&progress.measure, state.runtime.iteration());

        self.observe(&state, EngineEvent::Progress(progress.measure.clone()));

        Ok(vec![progress.measure])
    }

    fn observe(&self, state: &State<P::State>, stage: EngineEvent<<P::State as UserState>::Float>) {
        let state_view = StateView::new(state);
        self.observers.dispatch(P::NAME, state_view, &stage);
    }

    fn emit_checkpoint(&self, state: &State<P::State>, _reason: event::CheckpointReason) {
        if let Some(checkpoint_store) = self.checkpoint_store.as_ref() {
            let checkpoint = Checkpoint::new(state);
            if checkpoint_store.save(&checkpoint).is_err() {
                eprintln!("Failed to save checkpoint");
            }
            self.observe(&state, EngineEvent::CheckpointSaved);
        }
    }
}
