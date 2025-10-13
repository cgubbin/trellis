#![allow(dead_code)]

mod calculation;
mod controller;

pub mod prelude;
mod problem;
mod result;
mod runner;
mod watchers;

mod state;

pub use calculation::Calculation;
pub(crate) use controller::Control;

pub use problem::Problem;
pub use result::{CalculationResult, ErrorCause, Output, TrellisError};
pub use runner::GenerateBuilder;
pub use state::{Cause, ErrorEstimate, State, Status, UpdateData, UserState};
// pub use watchers::Tracer;
pub use watchers::{Frequency, Target};

pub use web_time::Duration;

pub trait TrellisFloat: std::fmt::Display + serde::Serialize {}

impl TrellisFloat for f32 {}
impl TrellisFloat for f64 {}
