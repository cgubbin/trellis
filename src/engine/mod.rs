mod builder;
mod cancellation;
mod checkpoint;
mod context;
mod event;
mod lifecycle;
mod policy;
mod result;
mod termination;

use crate::progress::{Progress, ProgressRow};
pub use builder::GenerateBuilder;
pub use cancellation::CancellationGuard;
pub use checkpoint::Checkpoint;
use context::EngineContext;
pub(crate) use event::{EngineAction, EventBatch};
pub(crate) use lifecycle::EngineStage;
use policy::{EnginePolicy, PolicyStack};
pub use result::{EngineFailure, EngineOutput, EngineResult};
pub use termination::Termination;

use num_traits::float::FloatCore;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::{
    watchers::{ObservationContext, Observers},
    Output, UserState,
};
use crate::{Problem, Procedure, State};

pub type Error = Box<dyn std::error::Error>;

/// General purpose calculation engine
pub struct Engine<P>
where
    P: Procedure,
    P::State: UserState,
{
    /// Procedure to be run
    procedure: P,
    /// The problem to solve
    problem: Problem<P::Problem>,

    policy: PolicyStack<<P::State as UserState>::Float>,
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
}

impl<P> Engine<P>
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore,
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
                EngineAction::Step => continue,
                EngineAction::Stop(reason) => {
                    state.runtime.terminate(reason);
                    return self.finalise(state);
                }
                EngineAction::EmitCheckpoint => self.observe(
                    &state,
                    state.runtime.iteration(),
                    None,
                    EngineStage::Checkpoint,
                    false,
                ),
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

        self.observe(
            &state,
            state.runtime.iteration(),
            None,
            EngineStage::Initialisation,
            false,
        );

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

        self.observe(
            &state,
            state.runtime.iteration(),
            None,
            EngineStage::WrapUp,
            true,
        );

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

        let ctx = crate::watchers::ObservationContext {
            iteration: state.runtime.iteration(),
            termination: None,
            stage: EngineStage::Iteration,
        };

        let progress = state.user.progress();

        state
            .convergence
            .observe(&progress.measure, state.runtime.iteration());

        self.observe(
            state,
            state.runtime.iteration(),
            None,
            EngineStage::Iteration,
            false,
        );

        Ok(vec![progress.measure])
    }

    fn observe(
        &self,
        state: &State<P::State>,
        iteration: usize,
        termination: Option<Termination>,
        stage: EngineStage,
        is_exit: bool,
    ) {
        let ctx = crate::watchers::ObservationContext {
            iteration,
            termination,
            stage,
        };
        self.observers
            .observe_progress(P::NAME, ProgressRow::from(state), &ctx, is_exit);
        self.observers.observe_state(P::NAME, &state, &ctx, is_exit);
    }
}

// // impl<P, Q> Engine<P, Q>
// // where
// // P: Procedure,
// // P::State: UserState,
// // <P::State as UserState>::Float: FloatCore,
// // Q: EnginePolicy<P::State, Float = <P::State as UserState>::Float>,
// // {
// //     #[instrument(name = "running trellis computation", fields(ident = P::NAME), skip_all)]
// //     pub fn run(mut self) -> EngineResult<P::Output, P::State, P::Error> {
// //         match self.run_inner() {
// //             Ok(result) => result,
// //             Err(result) => result,
// //         }
// //     }

// //     fn run_inner(
// //         &mut self,
// //     ) -> Result<
// //         EngineResult<P::Output, P::State, P::Error>,
// //         EngineResult<P::Output, P::State, P::Error>,
// //     > {
// //         let state = self.initialise_phase().map_err(|e| EngineResult::Failed {
// //             error: e,
// //             checkpoint: None,
// //         })?;

// //         let state = self
// //             .execution_phase(state)
// //             .map_err(|(e, s)| EngineResult::Failed {
// //                 error: e,
// //                 checkpoint: Some({ crate::Checkpoint { state: s } }),
// //             })?;

// //         Ok(self.finalisation_phase(state))
// //     }

// //     fn initialise_phase(&mut self) -> Result<State<P::State>, P::Error> {
// //         // Todo: Load checkpoints? (resuscitate)
// //         self.start_time = self.now();

// //         let mut state = self.state.take().unwrap();

// //         // TODO: This only really matters if there is a checkpoint loaded, at the moment we have
// //         // none so the check is redundant
// //         state = if !state.is_initialised() {
// //             self.initialise_state(state)?
// //         } else {
// //             state
// //         };

// //         Ok(state)
// //     }

// //     fn execution_phase(
// //         &mut self,
// //         mut state: State<P::State>,
// //     ) -> Result<State<P::State>, (P::Error, State<P::State>)> {
// //         loop {
// //             let prev = state.clone();
// //             let mut specific = self
// //                 .procedure
// //                 .step(
// //                     &mut self.problem,
// //                     state.take_specific(),
// //                     CancellationGuard {
// //                         token: &self.cancellation,
// //                     },
// //                 )
// //                 .map_err(|e| (e, prev))?;

// //             state.runtime.increment_iteration();
// //             specific.update();

// //             let progress = specific.progress();

// //             match self
// //                 .policy
// //                 .step(&mut state, progress, self.cancellation.is_cancelled())
// //             {
// //                 StepDecision::Continue => {
// //                     // nothing
// //                 }

// //                 StepDecision::Terminate(reason) => {
// //                     state.runtime.terminate(reason);
// //                     break;
// //                 }
// //                 StepDecision::Pass => {
// //                     todo!()
// //                 }
// //             }
// //         }
// //         Ok(state)
// //     }

// //     fn finalisation_phase(
// //         &mut self,
// //         mut state: State<P::State>,
// //     ) -> EngineResult<P::Output, P::State, P::Error> {
// //         // We can only get here if the procedure actually terminated, so we can unwrap
// //         let termination = state
// //             .termination()
// //             .expect("execution phase guarantees termination");

// //         let result = self.wrap_up(state);

// //         if let Err(e) = result {
// //             return EngineResult::Failed {
// //                 error: e,
// //                 checkpoint: None,
// //             };
// //         }

// //         let result = result.unwrap();

// //         if termination.failed() {
// //             return EngineResult::Terminated {
// //                 output: result,
// //                 termination,
// //             };
// //         }

// //         EngineResult::Success(result)
// //     }

// //     #[instrument(name = "initialising runner", fields(ident = P::NAME), skip_all)]
// //     fn initialise_state(
// //         &mut self,
// //         mut state: State<P::State>,
// //     ) -> Result<State<P::State>, P::Error> {
// //         let specific_state = self
// //             .procedure
// //             .initialise(&mut self.problem, state.take_specific())?;

// //         state = state.set_specific(specific_state).update();

// //         self.observers
// //             .update(P::NAME, &state, Stage::Initialisation);

// //         Ok(state)
// //     }

// //     // #[instrument(name = "performing iteration", fields(ident = P::NAME, iter = state.iteration()), skip_all)]
// //     // fn once(&mut self, mut state: State<P::State>) -> Result<State<P::State>, P::Error> {
// //     //     let mut specific = self
// //     //         .procedure
// //     //         .step(
// //     //             &mut self.problem,
// //     //             state.take_specific(),
// //     //             CancellationGuard {
// //     //                 token: &self.cancellation,
// //     //             },
// //     //         )
// //     //         .map_err(|e| (e, prev))?;

// //     //     state.runtime.increment_iteration();
// //     //     specific.update();

// //     //     let progress = specific.progress();

// //     //     match self
// //     //         .policy
// //     //         .step(&mut state, progress, self.cancellation.is_cancelled())
// //     //     {
// //     //         StepDecision::Continue => {
// //     //             // nothing
// //     //         }

// //     //         StepDecision::Terminate(reason) => {
// //     //             state.runtime.terminate(reason);
// //     //             break;
// //     //         }
// //     //     }
// //     // }

// //     #[instrument(name = "wrapping up runner", fields(ident = P::NAME), skip_all)]
// //     fn wrap_up(
// //         &mut self,
// //         mut state: State<P::State>,
// //     ) -> Result<Output<P::Output, P::State>, P::Error> {
// //         let result = self
// //             .procedure
// //             .finalise(&mut self.problem, state.take_specific())?;

// //         self.observers.update(P::NAME, &state, Stage::WrapUp);

// //         Ok(Output::new(result, state))
// //     }
// // }
