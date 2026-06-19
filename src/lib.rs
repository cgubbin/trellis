#![allow(dead_code)]

mod procedure;

mod engine;
pub mod prelude;
mod problem;
mod progress;
mod result;
mod watchers;

mod state;

pub use procedure::Procedure;

pub use engine::{CancellationGuard, GenerateBuilder, Termination};
pub use problem::Problem;
pub use result::{Output, TrellisError};
pub use state::{State, UserState};
pub use watchers::Frequency;

pub use web_time::Duration;

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
