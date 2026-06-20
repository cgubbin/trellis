//! # Trellis
//!
//! Trellis is a generic, event-driven numerical engine for iterative procedures.
//!
//! It provides a structured execution environment for algorithms that evolve a state over time,
//! producing progress signals, convergence diagnostics, and termination conditions.
//!
//! ## Core concepts
//!
//! ### Procedure
//! A [`Procedure`] defines the numerical algorithm being executed.
//! It operates over a user-defined [`UserState`] and emits [`Progress`] updates.
//!
//! ### Engine
//! The engine drives execution of a procedure, managing:
//! - iteration lifecycle
//! - runtime tracking
//! - convergence tracking
//! - cancellation
//! - observer dispatch
//!
//! ### State model
//! Each run maintains a [`State`] consisting of:
//! - user-defined state (`UserState`)
//! - runtime metadata (iterations, timing)
//! - convergence tracking (best/current metrics)
//!
//! ### Observers
//! Observers subscribe to engine events via [`watchers::Observe`] and receive:
//! - progress updates
//! - lifecycle events
//! - termination signals
//!
//! Observers are used for:
//! - logging and tracing
//! - CSV export
//! - plotting
//! - metrics aggregation
//!
//! ### Output model
//! A run produces an [`EngineOutput`] containing:
//! - the user result
//! - termination reason
//! - runtime summary
//! - optional snapshot (if enabled)
//!
//! ## Design philosophy
//!
//! Trellis is built around four principles:
//!
//! ### 1. Separation of concerns
//! - procedure logic is independent of execution
//! - observers are independent of engine logic
//!
//! ### 2. Streaming execution
//! The engine emits incremental [`EngineSignal`] events during execution.
//!
//! ### 3. Explicit state evolution
//! State is not hidden; it is tracked explicitly via [`State`] and [`StateView`].
//!
//! ### 4. Optional capabilities
//! Features like snapshotting are opt-in via traits rather than enforced globally.
//!
//! ## Floating-point abstraction
//!
//! All numeric computation is abstracted over [`TrellisFloat`], supporting `f32` and `f64`
//! with extensibility for custom numeric types.
//!
//! ## Example
//!
//! ```ignore
//! let result = MyProcedure::new()
//!     .build_for(problem)
//!     .time(true)
//!     .finalise()
//!     .run();
//! ```
#![allow(dead_code)]

mod procedure;

mod engine;
mod problem;
mod progress;
mod result;
mod watchers;

mod state;

pub use procedure::Procedure;

pub use engine::{
    CancellationGuard, GenerateBuilder, InMemoryCheckpointStore, JsonCheckpointStore, Termination,
};

pub use engine::{
    AbsoluteTolerancePolicy, CancellationPolicy, CheckpointPolicy, CompletionPolicy,
    MaxIterationPolicy, NoProgressPolicy, RelativeTolerancePolicy, StagnationPolicy,
    TargetValuePolicy, TimeoutPolicy,
};

pub use problem::Problem;

pub use result::{EngineOutput, EngineOutputWithSnapshot, TrellisError};

pub use state::{Snapshotable, StateRestorer, UserState};

pub use progress::{Progress, ProgressDiagnostics};

pub use watchers::{CsvProgressWriter, Frequency, Observe, PlotObserver, Tracer};

pub trait TrellisFloat:
    std::fmt::Display
    + std::fmt::Debug
    + serde::Serialize
    + serde::de::DeserializeOwned
    + num_traits::float::FloatCore
{
}

impl TrellisFloat for f32 {}
impl TrellisFloat for f64 {}
