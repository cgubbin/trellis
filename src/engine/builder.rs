//! # Engine Builder API
//!
//! This module provides a fluent builder for constructing and configuring an [`Engine`].
//!
//! The builder is responsible for assembling all runtime components:
//!
//! - numerical procedure (`Procedure`)
//! - initial state (`State`)
//! - policy stack (`PolicyStack`)
//! - observers (`Observe` implementations)
//! - cancellation handling
//! - optional checkpoint storage
//!
//! ## Design philosophy
//!
//! The builder is intentionally *immutable-by-transformation*:
//! each method consumes `self` and returns a modified builder.
//!
//! This ensures:
//! - predictable construction flow
//! - no shared mutable setup state
//! - easy composability of configurations
//!
//! ## Minimal usage
//!
//! ```ignore
//! let engine = MyProcedure::new()
//!     .build_for(problem)
//!     .finalise();
//! ```
//!
//! ## With configuration
//!
//! ```ignore
//! let engine = MyProcedure::new()
//!     .build_for(problem)
//!     .time(true)
//!     .with_default_policies(max_iter, absolute_tolerance)
//!     .and_policy(my_policy)
//!     .attach_observer(my_tracer, Frequency::Always)
//!     .finalise();
//! ```
//!
//! ## Checkpointing
//!
//! Checkpointing is optional and provided via a backend store:
//!
//! ```ignore
//! let engine = MyProcedure::new()
//!     .build_for(problem)
//!     .with_checkpoint_store(FileCheckpointStore::new(path))
//!     .finalise();
//! ```
//!
use num_traits::float::FloatCore;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::engine::policy::{EnginePolicy, PolicyStack};
use crate::{
    engine::{
        checkpoint::{CheckpointError, CheckpointStore, NoCheckpoint},
        Engine,
    },
    watchers::{Frequency, Observe, Observers},
    Procedure, State, UserState,
};

pub trait GenerateBuilder: Sized + Procedure
where
    Self::State: UserState,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self, NoCheckpoint>;
}

impl<P> GenerateBuilder for P
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self, NoCheckpoint> {
        Builder {
            procedure: self,
            problem,
            state: State::new(),
            time: true,
            cancellation_token: None,

            observers: Observers::new(),

            policies: PolicyStack::new(),

            checkpoint_store: None,
        }
    }
}

pub struct Builder<P, C>
where
    P: Procedure,
    P::State: UserState,
{
    procedure: P,
    problem: P::Problem,
    state: State<P::State>,
    time: bool,
    cancellation_token: Option<CancellationToken>,

    observers: Observers<P::State>,

    policies: PolicyStack<<P::State as UserState>::Float>,
    checkpoint_store: Option<C>,
}

impl<P> Builder<P, NoCheckpoint>
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore + 'static,
{
    #[must_use]
    pub fn with_checkpoint_store<C: CheckpointStore<P::State>>(
        mut self,
        checkpoint_store: C,
    ) -> Builder<P, C> {
        Builder {
            procedure: self.procedure,
            problem: self.problem,
            state: self.state,
            time: self.time,
            cancellation_token: self.cancellation_token,
            observers: self.observers,
            policies: self.policies,
            checkpoint_store: Some(checkpoint_store),
        }
    }
}

impl<P, C> Builder<P, C>
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
    pub fn attach_observer<OBS>(mut self, observer: OBS, frequency: Frequency) -> Self
    where
        OBS: Observe<P::State> + 'static,
    {
        self.observers
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

    #[must_use]
    pub fn with_default_policies(
        mut self,
        max_iter: usize,
        absolute_tolerance: <P::State as UserState>::Float,
    ) -> Self {
        self.policies = self
            .policies
            .merge(PolicyStack::standard(max_iter, absolute_tolerance));
        self
    }

    /// Finalise with default policy
    pub fn finalise(mut self) -> Engine<P, PolicyStack<<P::State as UserState>::Float>, C>
    where
        <P::State as UserState>::Float: num_traits::FromPrimitive,
    {
        let cancellation = self.cancellation_token.unwrap_or_default();

        #[cfg(feature = "ctrlc")]
        {
            let token = cancellation.clone();
            ctrlc::set_handler(move || {
                if let Some(t) = &token {
                    t.cancel();
                }
            })
            .unwrap();
        }

        Engine {
            procedure: self.procedure,
            problem: crate::Problem::new(self.problem),
            state: Some(self.state),

            time: self.time,
            start_time: None,

            cancellation,

            policy: self.policies,

            observers: self.observers,
            checkpoint_store: self.checkpoint_store,
        }
    }

    /// Finalise with custom policy
    pub fn finalise_with(
        self,
        policy: PolicyStack<<P::State as UserState>::Float>,
    ) -> Engine<P, PolicyStack<<P::State as UserState>::Float>, C> {
        let cancellation = self.cancellation_token.unwrap_or_default();

        #[cfg(feature = "ctrlc")]
        {
            let token = cancellation.clone();
            ctrlc::set_handler(move || {
                if let Some(t) = &token {
                    t.cancel();
                }
            })
            .unwrap();
        }

        Engine {
            procedure: self.procedure,
            problem: crate::Problem::new(self.problem),
            state: Some(self.state),

            time: self.time,
            start_time: None,

            cancellation,

            policy,

            observers: self.observers,
            checkpoint_store: self.checkpoint_store,
        }
    }

    pub fn with_checkpoint_resumed(mut self) -> Result<Self, CheckpointError>
    where
        C: CheckpointStore<P::State>,
    {
        if let Some(store) = &self.checkpoint_store {
            if let Some(checkpoint) = store.load()? {
                self.state = checkpoint.into_state();
            }
        }

        Ok(self)
    }
}
