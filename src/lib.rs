//! # Trellis
//!
//! Trellis is a generic, event-driven numerical engine for iterative procedures.
//!
//! It provides a structured execution environment for algorithms that evolve a state over time,
//! producing progress signals, convergence diagnostics, and termination conditions.
//!
//! # Policies
//!
//! Policies control solver execution.
//!
//! During a run, the engine collects progress information from the procedure and
//! passes it to one or more policies. Policies inspect this information and
//! decide whether the solver should:
//!
//! - continue running,
//! - terminate successfully,
//! - terminate early,
//! - or request some other engine action.
//!
//! Policies are the primary mechanism used to implement convergence criteria,
//! iteration limits, stagnation detection and timeout handling.
//!
//! ## Policies vs Observers
//!
//! Policies influence solver behaviour.
//!
//! Observers only observe solver behaviour.
//!
//! A policy may terminate a calculation. An observer may not.
//!
//! ```text
//! Progress ──► Policy ──► Engine Action
//!            │
//!            └────► Observer
//! ```
//!
//! ## Attaching Policies
//!
//! Policies are attached through the builder.
//!
//! ```rust
//! use trellis_runner::{CancellationGuard, MaxIterationPolicy, StagnationPolicy, GenerateBuilder};
//!
//! struct MyProcedure;
//! struct MyProblem;
//! struct MyState;
//!
//! impl trellis_runner::Procedure<MyProblem> for MyProcedure {
//!     const NAME: &'static str = "My Procedure";
//!     type State = MyState;
//!     type Output = ();
//!
//!     fn step(
//!         &self,
//!         _: &mut MyProblem,
//!         _: &mut Self::State,
//!         _guard: CancellationGuard<'_>,
//!     ) {
//!         ()
//!     }
//!
//!     fn finalise(&self, _: &mut MyProblem, _: &Self::State) {}
//! }
//!
//!
//! impl trellis_runner::UserState for MyState {
//!     type Float = f64;
//!
//!     fn progress(&self) -> trellis_runner::Progress<f64> {
//!         trellis_runner::Progress::Complete
//!     }
//! }
//!
//! let engine = MyProcedure
//!     .build_for(MyProblem)
//!     .with_initial_state(MyState)
//!     .and_policy(MaxIterationPolicy::new(10_000))
//!     .and_policy(StagnationPolicy::new(10))
//!     .finalise();
//! ```
//!
//! Multiple policies may be attached.
//!
//! The engine stops as soon as any policy requests termination.
//!
//! ## Built-in Policies
//!
//! Trellis provides several commonly useful policies.
//!
//! | Policy | Purpose |
//! |---------|----------|
//! | `MaxIterationPolicy` | Stop the engine after a fixed number of iterations |
//! | `TimeoutPolicy` | Stops the engine after a maximum wall-clock duration |
//! | `AbsoluteTolerancePolicy` | Stops the engine when the mean absolute error over a rolling window falls below a user-defined tolerance|
//! | `RelativeTolerancePolicy` | Stops the engine when the mean relative error over a rolling window falls below a user-defined tolerance|
//! | `StagnationPolicy` | Stops the engine when the improvement of the best observed value over a rolling window falls below a relative tolerance threshold |
//! | `NoProgressPolicy` | Stops the engine when the best observed objective value fails to improve by a relative tolerance for a specified number of consecutive iterations |
//! | `TargetValuePolicy` | Stops the engine when the mean absolute distance to a target value remains below a specified tolerance over a rolling window |
//! | `CheckpointPolicy` | Define frequency of checkpoint generation |
#![allow(dead_code)]

mod procedure;

mod engine;
mod progress;
mod result;
mod watchers;

mod state;

pub(crate) use procedure::Infallible;

pub use procedure::{FallibleProcedure, Procedure};

pub use engine::{
    AbsoluteTolerancePolicy, CancellationGuard, CheckpointPolicy, EngineFailure, GenerateBuilder,
    GenerateBuilderFallible, InMemoryCheckpointStore, MaxIterationPolicy, NoProgressPolicy,
    RelativeTolerancePolicy, StagnationPolicy, TargetValuePolicy, Termination, TimeoutPolicy,
};

#[cfg(feature = "writing")]
pub use engine::JsonCheckpointStore;

pub use result::{EngineOutput, EngineOutputWithSnapshot, RunSummary, TrellisError};

pub use state::{Snapshotable, StateRestorer, UserState};

pub use progress::{Progress, ProgressDiagnostics};

pub use watchers::{Frequency, Observe, Tracer};

#[cfg(feature = "writing")]
pub use watchers::CsvProgressWriter;

#[cfg(feature = "plotting")]
pub use watchers::PlotObserver;

pub trait TrellisFloat: std::fmt::Display + std::fmt::Debug + num_traits::float::FloatCore {}

impl TrellisFloat for f32 {}
impl TrellisFloat for f64 {}
