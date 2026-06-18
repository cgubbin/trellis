use std::time::{Duration, Instant};

pub(crate) struct EngineContext<'a> {
    pub(crate) cancelled: bool,
    pub(crate) iter: usize,
    pub(crate) elapsed: Duration,
    pub checkpoint_due: bool,
    pub start_time: Instant,
    pub _marker: std::marker::PhantomData<&'a ()>,
}
