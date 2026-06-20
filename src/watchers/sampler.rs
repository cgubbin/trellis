use crate::engine::EngineSignal;
use crate::state::StateView;
use crate::watchers::Observe;

/// Reduces event frequency by only forwarding every N iterations.
///
/// Useful for performance control and large-scale runs.
pub struct Sampler<S, O> {
    every: usize,
    inner: O,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, O> Sampler<S, O> {
    pub fn new(every: usize, inner: O) -> Self {
        Self {
            every,
            inner,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<S, O> Observe<S> for Sampler<S, O>
where
    S: crate::UserState + Send + Sync,
    O: Observe<S>,
{
    fn observe(
        &self,
        ident: &'static str,
        state: StateView<'_, S>,
        event: &EngineSignal<S::Float>,
    ) {
        if !state.iteration().is_multiple_of(self.every) {
            return;
        }

        self.inner.observe(ident, state, event);
    }
}
