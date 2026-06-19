use std::time::{Duration, Instant};

/// Immutable (from the perspective of policies) snapshot of engine runtime state.
///
/// `EngineContext` is the *only structured view of engine control state* independent of:
/// - numerical progress (`Progress`)
/// - event history (`EventBatch`)
/// - solver state (`State`)
///
/// It is passed into the policy layer to support decisions that are not
/// directly derivable from `Progress`, such as:
/// - cancellation
/// - wall-clock timeout
/// - iteration-based scheduling
/// - checkpoint scheduling
///
/// # Design intent
///
/// This type deliberately avoids exposing full engine state. Policies should
/// remain:
/// - deterministic (no hidden mutation)
/// - stateless or explicitly stateful via internal fields
/// - independent of solver implementation details
#[derive(Debug)]
pub struct EngineContext<'a> {
    /// Whether execution has been externally cancelled.
    ///
    /// Typically driven by a cancellation token or user request.
    pub(crate) cancelled: bool,

    /// Current iteration count.
    ///
    /// This is incremented once per engine loop cycle, regardless of how many
    /// internal solver steps occur.
    pub(crate) iter: usize,

    /// Wall-clock time elapsed since engine start.
    ///
    /// Used for timeout-based policies. This value is updated by the engine
    /// and not by policies.
    pub(crate) elapsed: Duration,

    /// Indicates whether a checkpoint is currently due according to engine
    /// scheduling logic (not policy logic).
    ///
    /// This allows the engine to coordinate scheduled checkpointing separately
    /// from policy-triggered checkpoint requests.
    pub checkpoint_due: bool,

    /// Absolute start time of the engine run.
    ///
    /// Provided for policies that require absolute timestamps rather than
    /// relative durations.
    pub start_time: Instant,

    pub _marker: std::marker::PhantomData<&'a ()>,
}

impl Default for EngineContext<'_> {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            iter: 0,
            cancelled: false,
            checkpoint_due: false,
            elapsed: Duration::default(),
            _marker: std::marker::PhantomData,
        }
    }
}
