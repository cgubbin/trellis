use num_traits::float::FloatCore;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::engine::policy::{EnginePolicy, PolicyStack};
use crate::{
    engine::Engine,
    watchers::{
        Frequency, Observers, ProgressObserver, ProgressObservers, StateObserver, StateObservers,
    },
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
    <P::State as UserState>::Float: FloatCore,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self> {
        Builder {
            procedure: self,
            problem,
            state: State::new(),
            time: true,
            cancellation_token: None,

            state_observers: StateObservers::new(),
            progress_observers: ProgressObservers::new(),

            policies: PolicyStack::new(),
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

    state_observers: StateObservers<P::State>,
    progress_observers: ProgressObservers<<P::State as UserState>::Float>,

    policies: PolicyStack<<P::State as UserState>::Float>,
}

impl<P> Builder<P>
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore + 'static,
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
        OBS: StateObserver<P::State> + 'static,
    {
        self.state_observers
            .attach(Arc::new(Mutex::new(observer)), frequency);
        self
    }

    /// Attach a progress observer (lightweight per-iteration stream)
    #[must_use]
    pub fn attach_progress_observer<OBS>(mut self, observer: OBS, frequency: Frequency) -> Self
    where
        OBS: ProgressObserver<<P::State as UserState>::Float> + 'static,
    {
        self.progress_observers
            .attach(Arc::new(Mutex::new(observer)), frequency);
        self
    }

    #[must_use]
    pub fn and_policy<Q>(mut self, policy: Q) -> Self
    where
        Q: EnginePolicy<<P::State as UserState>::Float> + 'static,
    {
        self.policies = self.policies.add(policy);
        self
    }

    #[must_use]
    pub fn cancellation_token(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
        self
    }

    /// Finalise with default policy
    pub fn finalise(self) -> Engine<P>
    where
        <P::State as UserState>::Float: num_traits::FromPrimitive,
    {
        let policies = if self.policies.is_empty() {
            PolicyStack::default(
                1000,
                <<P::State as UserState>::Float as num_traits::FromPrimitive>::from_f64(1e-7)
                    .unwrap(),
            )
        } else {
            PolicyStack::new()
        };
        self.finalise_with(policies)
    }

    /// Finalise with custom policy
    pub fn finalise_with(self, policy: PolicyStack<<P::State as UserState>::Float>) -> Engine<P> {
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

            policy: self.policies.merge(policy),

            observers: Observers::from_parts(self.progress_observers, self.state_observers),
        }
    }
}
