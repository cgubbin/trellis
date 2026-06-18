mod builder;
mod cancellation;
mod checkpoint;
mod event;
mod policy;
mod result;
mod termination;

pub use builder::GenerateBuilder;
pub use cancellation::CancellationGuard;
pub(crate) use event::{EngineEvent, RawEvent};
use policy::EnginePolicy;
pub use result::{EngineFailure, EngineOutput, EngineResult};
pub use termination::Termination;

use num_traits::float::FloatCore;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use web_time::{Duration, Instant};

use crate::{
    watchers::{Observable, ObserverSlice, ObserverVec, Stage},
    Output, UserState,
};
use crate::{Problem, Procedure, State};

pub type Error = Box<dyn std::error::Error>;

/// General purpose calculation engine
pub struct Engine<P, Q>
where
    P: Procedure,
    P::State: UserState,
    Q: EnginePolicy<P::State>,
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

    observers: ObserverVec<State<P::State>>,
}

impl<P, Q> Engine<P, Q>
where
    P: Procedure,
    P::State: UserState,
    Q: EnginePolicy<P::State>,
{
    fn now(&self) -> Option<Instant> {
        if self.time {
            return Some(Instant::now());
        }
        None
    }

    pub(crate) fn observers(&self) -> ObserverSlice<'_, State<P::State>> {
        self.observers.as_slice()
    }

    pub(crate) fn observers_mut(&mut self) -> &mut ObserverVec<State<P::State>> {
        &mut self.observers
    }

    fn duration_since(&self, maybe_previous: Option<&Instant>) -> Option<Duration> {
        if let Some(previous) = maybe_previous {
            let now = self.now().unwrap();
            return Some(now.duration_since(*previous));
        }
        None
    }
}

impl<P, Q> Engine<P, Q>
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore,
    Q: EnginePolicy<P::State>,
{
    pub fn run(mut self) -> EngineResult<P::Output, P::State, P::Error> {
        let mut state = self.state.take().unwrap();

        self.initialise_state(&mut state)
            .map_err(|error| EngineFailure {
                error,
                state: state.clone(),
            })?;

        loop {
            self.step_once(&mut state)
                .map_err(|(error, state)| EngineFailure { error, state })?;

            let progress = state.user.progress();

            let cancelled = self.cancellation.is_cancelled();

            let action = self.policy.next(&state, progress, cancelled);

            match action {
                EngineEvent::Pass => continue,
                EngineEvent::CheckpointRequested => self.save_checkpoint(&state).unwrap(),
                EngineEvent::TerminationRequested(reason) => {
                    state.runtime.terminate(reason);
                    return self.finalise(state);
                }
                _ => unimplemented!("enum form not finalised..."),
            }
        }
    }

    #[instrument(name = "initialising runner", fields(ident = P::NAME), skip_all)]
    fn initialise_state(&mut self, state: &mut State<P::State>) -> Result<(), P::Error> {
        self.procedure
            .initialise(&mut self.problem, &mut state.user)?;

        self.observers
            .update(P::NAME, &state, Stage::Initialisation);

        Ok(())
    }

    #[instrument(name = "saving checkpoint", fields(ident = P::NAME), skip_all)]
    fn save_checkpoint(&mut self, state: &State<P::State>) -> Result<(), P::Error> {
        todo!()
    }

    #[instrument(name = "wrapping up runner", fields(ident = P::NAME), skip_all)]
    fn finalise(
        &mut self,
        mut state: State<P::State>,
    ) -> EngineResult<P::Output, P::State, P::Error> {
        let output = self
            .procedure
            .finalise(&mut self.problem, &mut state.user)
            .map_err(|e| EngineFailure {
                error: e,
                state: state.clone(),
            })?;

        self.observers.update(P::NAME, &state, Stage::WrapUp);

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
    ) -> Result<(), (P::Error, State<P::State>)> {
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

        let events = state
            .convergence
            .observe(progress.measure, state.runtime.iteration());

        for event in events {
            state.user.on_event(event);
        }

        Ok(())
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
