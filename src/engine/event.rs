use crate::progress::Progress;
use crate::Termination;


pub struct EventBatch<F> {
    pub events: Vec<Progress<F>>,
}

pub enum EngineAction {
    Continue,

    Step,

    EmitCheckpoint,

    Stop(Termination),
}
