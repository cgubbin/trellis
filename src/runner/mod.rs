mod builder;
mod killswitch;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use num_traits::float::FloatCore;
use tracing::instrument;
use web_time::{Duration, Instant};

use crate::{
    controller::{set_handler, Control},
    result::{CalculationResult, ErrorCause, TrellisError},
    watchers::{Observable, ObserverSlice, ObserverVec, Stage},
    Output, UserState,
};
use crate::{Calculation, Cause, Problem, State};
pub use builder::GenerateBuilder;
use killswitch::{Holder, Killswitch};

pub type Error = Box<dyn std::error::Error>;

/// General purpose calculation runner
pub struct Runner<C, P, S, T>
where
    C: Calculation<P, S>,
    S: UserState,
{
    /// Calculation to be run
    calculation: C,
    /// The problem to solve
    problem: Problem<P>,
    /// Current state of the run
    state: Option<State<S>>,
    /// Should execution be timed
    time: bool,
    /// Receiver
    ///
    /// When a signal is received on this channel the calculation is terminated.
    cancellation_token: Option<T>,
    /// Signals to terminate the calculation
    signals: Vec<Killswitch>,
    observers: ObserverVec<State<S>>,
}

impl<C, P, S, T> Runner<C, P, S, T>
where
    C: Calculation<P, S>,
    S: UserState,
{
    fn now(&self) -> Option<Instant> {
        if self.time {
            return Some(Instant::now());
        }
        None
    }

    pub(crate) fn observers(&self) -> ObserverSlice<'_, State<S>> {
        self.observers.as_slice()
    }

    pub(crate) fn observers_mut(&mut self) -> &mut ObserverVec<State<S>> {
        &mut self.observers
    }

    fn duration_since(&self, maybe_previous: Option<&Instant>) -> Option<Duration> {
        if let Some(previous) = maybe_previous {
            let now = self.now().unwrap();
            return Some(now.duration_since(*previous));
        }
        None
    }

    #[cfg(feature = "ctrlc")]
    fn initialise_control_c_handler(&mut self) -> Result<Arc<AtomicBool>, Error> {
        // Create an atomic bool: when a signal is received from ctrl c this will flip to true
        let received_kill_signal_from_control_c = Arc::new(AtomicBool::new(false));

        {
            // Clone the state as the value needs to move into the closure
            let state = received_kill_signal_from_control_c.clone();
            // Set the ctrl c handler. This ensures that when a signal is recorded by ctrl c the
            // state is flipped to true
            ctrlc::set_handler(move || {
                state.store(true, Ordering::SeqCst);
            })?;
        }

        Ok(received_kill_signal_from_control_c)
    }
}

impl<C, P, S, R> Runner<C, P, S, R>
where
    C: Calculation<P, S>,
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    fn kill_signal_received(&self) -> bool {
        self.signals.iter().any(|signal| signal.is_dead())
    }

    fn kill_cause(&self) -> Option<Cause> {
        self.signals
            .iter()
            .find(|signal| signal.is_dead())
            .map(|signal| signal.held_by().into())
    }

    #[instrument(name = "initialising runner", fields(ident = C::NAME), skip_all)]
    fn initialise(&mut self, mut state: State<S>) -> Result<State<S>, C::Error> {
        let specific_state = self
            .calculation
            .initialise(&mut self.problem, state.take_specific())?;

        state = state.set_specific(specific_state).update();

        self.observers
            .update(C::NAME, &state, Stage::Initialisation);

        Ok(state)
    }

    #[instrument(name = "performing iteration", fields(ident = C::NAME, iter = state.current_iteration()), skip_all)]
    fn once(
        &mut self,
        mut state: State<S>,
        maybe_start_time: Option<&Instant>,
    ) -> Result<State<S>, C::Error> {
        let _maybe_iteration_start_time = self.now().unwrap();

        let specific = self
            .calculation
            .next(&mut self.problem, state.take_specific())?;
        state = state.set_specific(specific);

        if let Some(total_duration) = self.duration_since(maybe_start_time) {
            state.record_time(total_duration);
        }
        state.increment_iteration();
        state = state.update();

        self.observers.update(C::NAME, &state, Stage::Iteration);

        Ok(state)
    }

    #[instrument(name = "wrapping up runner", fields(ident = C::NAME), skip_all)]
    fn wrap_up(&mut self, mut state: State<S>) -> Result<Output<C::Output, S>, C::Error> {
        let result = self
            .calculation
            .finalise(&mut self.problem, state.take_specific())?;

        self.observers.update(C::NAME, &state, Stage::WrapUp);

        Ok(Output::new(result, state))
    }

    /// Execute the runner
    #[instrument(name = "running trellis computation", fields(ident = C::NAME), skip_all)]
    pub fn run(mut self) -> CalculationResult<C::Output, S, C::Error> {
        // Todo: Load checkpoints? (resuscitate)
        let start_time = self.now();

        let mut state = self.state.take().unwrap();

        // TODO: This only really matters if there is a checkpoint loaded, at the moment we have
        // none so the check is redundant
        state = if !state.is_initialised() {
            self.initialise(state)?
        } else {
            state
        };

        loop {
            if self.kill_signal_received() {
                state = state.terminate_due_to(self.kill_cause().unwrap());
            }

            if state.is_terminated() {
                break;
            }
            state = self.once(state, start_time.as_ref())?;
        }

        // We can only get here if the calculation actually terminated, so we can unwrap
        let cause = match state.termination_cause() {
            Some(Cause::ControlC) => Some(ErrorCause::ControlC),
            Some(Cause::Parent) => Some(ErrorCause::CancellationToken),
            Some(Cause::Converged) => None,
            Some(Cause::ExceededMaxIterations) => Some(ErrorCause::MaxIterExceeded),
            None => unreachable!("the loop can only terminate if the state was actually converged"),
        };

        let result = self.wrap_up(state)?;

        if let Some(cause) = cause {
            return Err(TrellisError {
                cause,
                result: Some(result.result),
            });
        }

        Ok(result)
    }
}

impl<C, P, S, R> Runner<C, P, S, R>
where
    C: Calculation<P, S>,
    S: UserState,
    R: Control + 'static,
{
    fn initialise_kill_signal_handler(&mut self) -> Result<Arc<AtomicBool>, Error> {
        let received_kill_signal_from_controller = Arc::new(AtomicBool::new(false));

        // Clone the state as the value needs to move into the closure
        let state = received_kill_signal_from_controller.clone();
        set_handler(self.cancellation_token.take().unwrap(), move || {
            state.store(true, Ordering::SeqCst);
        })?;

        Ok(received_kill_signal_from_controller)
    }
}

pub trait InitialiseRunner {
    fn initialise_controllers(&mut self) -> Result<(), Error>;
}

impl<C, P, S> InitialiseRunner for Runner<C, P, S, ()>
where
    C: Calculation<P, S>,
    S: UserState,
{
    fn initialise_controllers(&mut self) -> Result<(), Error> {
        #[cfg(feature = "ctrlc")]
        {
            let received_kill_signal_from_control_c =
                Killswitch::new(Holder::CtrlC, self.initialise_control_c_handler()?);
            self.signals = vec![received_kill_signal_from_control_c];
        }
        Ok(())
    }
}

impl<C, P, S, R> InitialiseRunner for Runner<C, P, S, R>
where
    C: Calculation<P, S>,
    S: UserState,
    R: Control + 'static,
{
    fn initialise_controllers(&mut self) -> Result<(), Error> {
        #[cfg(feature = "ctrlc")]
        {
            let received_kill_signal_from_control_c =
                Killswitch::new(Holder::CtrlC, self.initialise_control_c_handler()?);
            self.signals = vec![received_kill_signal_from_control_c];
        }

        let received_kill_signal_from_controller =
            Killswitch::new(Holder::Parent, self.initialise_kill_signal_handler()?);
        self.signals.push(received_kill_signal_from_controller);
        Ok(())
    }
}
