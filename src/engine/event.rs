use crate::progress::Progress;
use crate::Termination;

/// A batch of progress signals emitted during a single solver iteration.
///
/// The engine aggregates low-level `Progress` signals produced by the
/// procedure and convergence subsystem into a single batch.
///
/// Policies consume this batch to make decisions about:
/// - convergence
/// - stagnation
/// - checkpointing
/// - termination
#[derive(Debug)]
pub struct EventBatch<F> {
    /// All progress signals emitted during the current iteration.
    pub events: Vec<Progress<F>>,
}

impl<F> EventBatch<F> {
    /// Generate and empty batch
    pub fn new() -> Self {
        Self { events: vec![] }
    }
    /// Adds a progress event to the batch.
    pub fn add(mut self, event: Progress<F>) -> Self {
        self.events.push(event);
        self
    }
}

/// Action returned by the policy layer after evaluating an [`EventBatch`]
/// and the current engine context.
///
/// This is the *only control signal* that influences the engine loop.
///
/// The engine reacts deterministically to this value.
#[derive(Clone, Debug, PartialEq)]
pub enum EngineAction {
    /// Continue normal execution with no side effects.
    Continue,

    /// Request that the engine persists a checkpoint of the current state.
    ///
    /// This does not stop execution; it is a side-effect request.
    EmitCheckpoint(CheckpointReason),

    /// Request termination of the solver.
    ///
    /// This immediately ends execution and propagates the termination reason
    /// to the final [`EngineOutput`].
    Stop(Termination),
}

/// Reason for emitting a checkpoint.
///
/// This distinguishes between scheduled persistence and semantic triggers
/// (e.g. stagnation recovery or user-driven requests).
#[derive(Clone, Debug, PartialEq)]
pub enum CheckpointReason {
    /// Checkpoint triggered on a fixed schedule (e.g. every N iterations).
    Scheduled,

    /// Checkpoint triggered due to stagnation detection.
    ///
    /// Typically used for recovery or restart strategies.
    Stagnation,

    /// Checkpoint triggered by an external user or system request.
    UserRequest,
}

/// High-level lifecycle events emitted by the engine.
///
/// These events are used for:
/// - observers / logging
/// - external monitoring
/// - UI updates
///
/// They are distinct from [`Progress`], which represents *numerical solver signals*.
pub enum EngineSignal<F> {
    /// Engine has completed initialisation and is ready to iterate.
    Initialised,

    /// A single progress signal emitted during iteration.
    Progress(Progress<F>),

    /// A checkpoint has been successfully persisted.
    CheckpointSaved,

    /// A checkpoint has been requested.
    CheckpointRequested(CheckpointReason),

    /// Engine has terminated for any reason.
    Termination(Termination),
}

impl<F> EngineSignal<F> {
    /// Returns a stable string tag identifying the event kind.
    pub fn as_tag(&self) -> &'static str {
        match self {
            Self::Initialised => "initialised",
            Self::Progress(_) => "progress",
            Self::CheckpointSaved => "checkpoint_saved",
            Self::Termination(_) => "termination",
            Self::CheckpointRequested(_) => "checkpoint_requested",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_batch_accumulates_events() {
        let batch = EventBatch::new()
            .add(Progress::Measure(1.0))
            .add(Progress::Measure(2.0));

        assert_eq!(batch.events.len(), 2);
    }
}
