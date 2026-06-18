/// An [`EngineAction`] is the result of the application of policy
use crate::Termination;

#[derive(Debug, Clone)]
pub enum PolicyDecision {
    Pass,
    Stop(Termination),
    SaveCheckpoint,
}
