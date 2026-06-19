#[derive(Copy, Clone, PartialEq)]
pub(crate) enum EngineStage {
    Initialisation,
    Iteration,
    Checkpoint,
    WrapUp,
}
