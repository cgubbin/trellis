#![allow(dead_code)]

mod controller;
mod procedure;

mod checkpoint;
mod engine;
pub mod prelude;
mod problem;
mod result;
mod watchers;

mod state;

pub use checkpoint::Checkpoint;
pub(crate) use controller::Control;
pub use procedure::Procedure;

pub use engine::{CancellationGuard, GenerateBuilder};
pub use problem::Problem;
pub use result::{EngineResult, Output, TrellisError};
pub use state::{ErrorEstimate, State, Status, Termination, UpdateData, UserState};
// pub use watchers::Tracer;
pub use watchers::{Frequency, Target};

pub use web_time::Duration;

pub trait TrellisFloat:
    std::fmt::Display + serde::Serialize + num_traits::float::FloatCore
{
}

impl TrellisFloat for f32 {}
impl TrellisFloat for f64 {}
