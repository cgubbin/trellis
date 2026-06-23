//! # Engine Builder API
//!
//! This module provides a fluent, consuming builder for constructing an [`Engine`] instance.
//!
//! The builder is responsible for assembling all runtime components required to execute a
//! procedure:
//!
//! - numerical procedure (`FallibleProcedure`)
//! - initial solver state (`State`)
//! - policy stack (`PolicyStack`)
//! - observers (`Observe` implementations)
//! - cancellation support (`CancellationToken`)
//! - execution extensions (`EngineSink`), including checkpointing
//!
//! ## Design philosophy
//!
//! The builder follows a *consuming accumulation model*:
//!
//! - Each method takes ownership of `self`
//! - Each call returns a modified builder
//! - No shared mutable setup state exists
//!
//! This ensures:
//! - deterministic construction order
//! - composable configuration layers
//! - separation between configuration and execution
//!
//! ## Execution model
//!
//! The engine operates on three independent layers:
//!
//! ### 1. Policies
//! Policies inspect solver progress and produce an [`EngineAction`]:
//! - continue execution
//! - request checkpointing
//! - stop execution
//!
//! Policies are composed in a [`PolicyStack`].
//!
//! ### 2. Observers
//! Observers receive structured state snapshots (`StateView`) and engine signals
//! for logging, monitoring, or metrics.
//!
//! ### 3. Extensions
//! Extensions react to high-level engine signals (`EngineSignal`) and perform
//! side effects such as:
//! - checkpoint persistence
//! - external storage
//! - asynchronous logging pipelines
//!
//! Extensions are decoupled from core execution logic.
//!
//! ## Checkpointing
//!
//! Checkpointing is implemented as an optional extension.
//! It is only available when the state type supports snapshotting (`Snapshotable`).
//!
//! Checkpoints are triggered by policies and handled by an `EngineSink` extension.
//!
//! ## Minimal usage
//!
//! ```ignore
//! let engine = MyFallibleProcedure::new()
//!     .build_for(problem)
//!     .finalise();
//! ```
//!
//! ## Fully configured usage
//!
//! ```ignore
//! let engine = MyFallibleProcedure::new()
//!     .build_for(problem)
//!     .time(true)
//!     .with_default_policies(max_iter, tol)
//!     .and_policy(my_policy)
//!     .attach_observer(tracer, Frequency::Always)
//!     .with_checkpoint_backend(store)
//!     .finalise();
//! ```
//!
//! ## Design note
//!
//! The builder does not enforce a single “correct” policy set.
//! Policies are always composed explicitly by the user or via helpers.
//!
use num_traits::float::FloatCore;
use std::sync::{Arc, Mutex};
use tokio_util::sync::CancellationToken;

use crate::engine::policy::{CancellationPolicy, CompletionPolicy, EnginePolicy, PolicyStack};
use crate::{
    engine::{
        checkpoint::{CheckpointBackend, CheckpointExtension},
        extensions::Extensions,
        Engine,
    },
    state::{Snapshotable, State, StateRestorer},
    watchers::{Frequency, Observe, Observers},
    FallibleProcedure, UserState,
};

pub trait GenerateBuilder: Sized {
    fn build_for<P>(self, problem: P) -> Builder<Self, P, Uninitialised>
    where
        Self: FallibleProcedure<P>,
        Self::State: UserState;
}

impl<Proc> GenerateBuilder for Proc {
    fn build_for<P>(self, problem: P) -> Builder<Self, P, Uninitialised>
    where
        Proc: FallibleProcedure<P>,
        Proc::State: UserState,
    {
        Builder {
            procedure: self,
            problem,
            state: None,
            time: true,
            cancellation_token: None,

            observers: Observers::new(),

            policies: PolicyStack::new()
                .add(CancellationPolicy)
                .add(CompletionPolicy),

            extensions: Extensions::new(),

            _initialised: std::marker::PhantomData,
        }
    }
}

pub struct Uninitialised;
pub struct Initialised;

pub struct Builder<Proc, P, I>
where
    Proc: FallibleProcedure<P>,
    Proc::State: UserState,
    <Proc::State as UserState>::Float: FloatCore,
{
    procedure: Proc,
    problem: P,
    state: Option<Proc::State>,
    time: bool,
    cancellation_token: Option<CancellationToken>,

    observers: Observers<Proc::State>,

    policies: PolicyStack<<Proc::State as UserState>::Float>,
    extensions: Extensions<Proc::State>,

    _initialised: std::marker::PhantomData<I>,
}

impl<Proc, P, I> Builder<Proc, P, I>
where
    Proc: FallibleProcedure<P>,
    Proc::State: UserState,
    <Proc::State as UserState>::Float: FloatCore + 'static,
{
    #[must_use]
    pub fn time(mut self, time: bool) -> Self {
        self.time = time;
        self
    }

    /// Attach a state observer (full state + stage awareness)
    #[must_use]
    pub fn attach_observer<OBS>(mut self, observer: OBS, frequency: Frequency) -> Self
    where
        OBS: Observe<Proc::State> + 'static,
    {
        self.observers
            .attach(Arc::new(Mutex::new(observer)), frequency);
        self
    }

    #[must_use]
    pub fn and_policy<Q>(mut self, policy: Q) -> Self
    where
        Q: EnginePolicy<<Proc::State as UserState>::Float> + 'static,
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
    /// Appends a standard policy set to the existing policy stack.
    ///
    /// This does not replace existing policies; it merges them into the current stack.
    pub fn with_default_policies(
        mut self,
        max_iter: usize,
        absolute_tolerance: <Proc::State as UserState>::Float,
        window_size: usize,
    ) -> Self {
        self.policies = self.policies.merge(PolicyStack::standard(
            max_iter,
            absolute_tolerance,
            window_size,
        ));
        self
    }

    #[must_use]
    /// Enables checkpointing support for this engine.
    ///
    /// This method is only available if the procedure state implements `Snapshotable`.
    ///
    /// When enabled, checkpoints are emitted via the engine extension system.
    pub fn with_checkpoint_backend<C>(mut self, store: C) -> Self
    where
        C: CheckpointBackend<
                <Proc::State as Snapshotable>::Snapshot,
                <Proc::State as UserState>::Float,
            > + 'static,
        Proc::State: Snapshotable,
    {
        self.extensions = self.extensions.add(CheckpointExtension::new(store));
        self
    }
}

impl<Proc, P> Builder<Proc, P, Uninitialised>
where
    Proc: FallibleProcedure<P>,
    Proc::State: UserState,
    <Proc::State as UserState>::Float: FloatCore + 'static,
{
    // TODO: Possibly unneeded if a valid state is always constructed in the initialise method
    #[must_use]
    pub fn with_initial_state(self, user: Proc::State) -> Builder<Proc, P, Initialised> {
        Builder {
            procedure: self.procedure,
            problem: self.problem,
            state: Some(user),
            time: self.time,
            cancellation_token: self.cancellation_token,

            observers: self.observers,

            policies: self.policies,

            extensions: self.extensions,

            _initialised: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn resume_from_checkpoint(
        self,
        snapshot: <Proc::State as Snapshotable>::Snapshot,
    ) -> Builder<Proc, P, Initialised>
    where
        Proc: FallibleProcedure<P>,
        Proc::State: Snapshotable + StateRestorer<Proc::State>,
    {
        let user = Proc::State::restore(snapshot);

        Builder {
            procedure: self.procedure,
            problem: self.problem,
            state: Some(user),
            time: self.time,
            cancellation_token: self.cancellation_token,

            observers: self.observers,

            policies: self.policies,

            extensions: self.extensions,

            _initialised: std::marker::PhantomData,
        }
    }
}

impl<Proc, P> Builder<Proc, P, Initialised>
where
    Proc: FallibleProcedure<P>,
    Proc::State: UserState,
    <Proc::State as UserState>::Float: FloatCore + 'static,
{
    /// Finalises the builder using the currently configured policy stack.
    ///
    /// If no policies were added, the engine will run with an empty policy stack
    /// (i.e. no termination conditions beyond external cancellation).efault policy
    pub fn finalise(mut self) -> Engine<Proc, P, PolicyStack<<Proc::State as UserState>::Float>>
    where
        <Proc::State as UserState>::Float: num_traits::FromPrimitive,
    {
        let user = self.state.take().expect("builder invariant: user is set");

        let cancellation = self.cancellation_token.unwrap_or_default();

        #[cfg(feature = "ctrlc")]
        {
            let token = cancellation.clone();
            ctrlc::set_handler(move || {
                token.cancel();
            })
            .unwrap();
        }

        Engine {
            procedure: self.procedure,
            problem: self.problem,
            state: State::new(user),

            time: self.time,
            start_time: None,

            cancellation,

            policy: self.policies,

            observers: self.observers,
            extensions: self.extensions,
        }
    }

    /// Finalises the engine with a custom policy stack.
    ///
    /// This replaces the builder’s internal policy stack but preserves:
    /// - observers
    /// - extensions
    /// - cancellation token
    /// - state configuration
    pub fn finalise_with(
        mut self,
        policy: PolicyStack<<Proc::State as UserState>::Float>,
    ) -> Engine<Proc, P, PolicyStack<<Proc::State as UserState>::Float>> {
        let user = self.state.take().expect("builder invariant: user is set");
        let cancellation = self.cancellation_token.unwrap_or_default();

        #[cfg(feature = "ctrlc")]
        {
            let token = cancellation.clone();
            ctrlc::set_handler(move || {
                token.cancel();
            })
            .unwrap();
        }

        Engine {
            procedure: self.procedure,
            problem: self.problem,
            state: State::new(user),

            time: self.time,
            start_time: None,

            cancellation,

            policy,

            observers: self.observers,
            extensions: self.extensions,
        }
    }
}
//     pub fn with_checkpoint_resumed(mut self) -> Result<Self, CheckpointError>
//     where
//         C: CheckpointBackend<Proc::State>,
//     {
//         if let Some(store) = &self.checkpoint_store {
//             if let Some(checkpoint) = store.load()? {
//                 self.state = checkpoint.into_state();
//             }
//         }

//         Ok(self)
//     }
// }
