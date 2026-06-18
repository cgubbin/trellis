use num_traits::float::FloatCore;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::policy::{DefaultPolicy, EnginePolicy};
use crate::{
    watchers::{Frequency, ObserverVec, ProgressObserver, StateObserver},
    Procedure, State, UserState,
};

pub trait GenerateBuilder: Sized + Procedure
where
    Self::State: UserState,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self>;
}

impl<P> GenerateBuilder for P
where
    P: Procedure,
    P::State: UserState,
    P::State::Float: FloatCore,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self> {
        Builder {
            procedure: self,
            problem,
            state: State::new(),
            time: true,
            cancellation_token: None,

            state_observers: ObserverVec::default(),
            progress_observers: Vec::new(),
        }
    }
}

pub struct Builder<P>
where
    P: Procedure,
    P::State: UserState,
{
    procedure: P,
    problem: P::Problem,
    state: State<P::State>,
    time: bool,
    cancellation_token: Option<CancellationToken>,

    state_observers: ObserverVec<State<P::State>>,
    progress_observers: Vec<Arc<dyn ProgressObserver<P::State::Float>>>,
}

impl<P> Builder<P>
where
    P: Procedure,
    P::State: UserState,
{
    #[must_use]
    pub fn time(mut self, time: bool) -> Self {
        self.time = time;
        self
    }

    /// Apply runtime configuration to the initial state.
    #[must_use]
    pub fn configure<F>(mut self, f: F) -> Self
    where
        F: FnOnce(State<P::State>) -> State<P::State>,
    {
        self.state = f(self.state);
        self
    }

    /// Attach a state observer (full state + stage awareness)
    #[must_use]
    pub fn attach_state_observer<OBS>(mut self, observer: OBS, frequency: Frequency) -> Self
    where
        OBS: StateObserver<State<P::State>> + 'static,
    {
        self.state_observers
            .attach(Arc::new(Mutex::new(observer)), frequency);
        self
    }

    /// Attach a progress observer (lightweight per-iteration stream)
    #[must_use]
    pub fn attach_progress_observer<OBS>(mut self, observer: OBS) -> Self
    where
        OBS: ProgressObserver<P::State::Float> + 'static,
    {
        self.progress_observers.push(Arc::new(observer));
        self
    }

    #[must_use]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    /// Finalise with default policy
    pub fn finalise(self) -> Engine<P, DefaultPolicy<P::State::Float>>
    where
        DefaultPolicy<P::State::Float>: EnginePolicy<P::State::Float>,
    {
        self.finalise_with(DefaultPolicy::default())
    }

    /// Finalise with custom policy
    pub fn finalise_with<Q>(self, policy: Q) -> Engine<P, Q>
    where
        Q: EnginePolicy<P::State::Float>,
    {
        let cancellation = self
            .cancellation_token
            .unwrap_or_else(CancellationToken::new);

        #[cfg(feature = "ctrlc")]
        {
            let _ = crate::controller::attach_ctrlc(cancellation.clone());
        }

        Engine {
            procedure: self.procedure,
            problem: crate::Problem::new(self.problem),
            state: Some(self.state),

            time: self.time,
            start_time: None,

            cancellation,

            policy,

            state_observers: self.state_observers,
            progress_observers: self.progress_observers,
        }
    }
}
