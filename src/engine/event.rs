use crate::progress::Progress;
use crate::Termination;

use std::time::Duration;

pub struct EventBatch<F> {
    pub events: Vec<Progress<F>>,
}

pub enum EngineAction {
    Continue,

    Step,

    Stop(Termination),
}
