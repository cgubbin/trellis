// engine_action.rs
use crate::Termination;

#[derive(Debug, Clone)]
pub enum EngineAction {
    Initialise,
    Step,
    Finalise,

    /// Continue execution without special action
    Continue,

    /// Stop immediately with a reason
    Stop(Termination),
}
