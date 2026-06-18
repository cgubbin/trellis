use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::ProgressReport,
    state::{State, UserState},
};

pub struct MaxIterationPolicy {
    max_iters: usize,
}

impl MaxIterationPolicy {
    pub fn new(max_iters: usize) -> Self {
        Self { max_iters }
    }
}

impl<S> EnginePolicy<S> for MaxIterationPolicy
where
    S: UserState,
{
    fn next(
        &mut self,
        _state: &State<S>,
        events: &[RawEvent<S::Float>],
        cancelled: bool,
    ) -> EngineEvent<S::Float> {
        for each in events {
            match *each {
                RawEvent::Iteration { iter } if iter > self.max_iters => {
                    return EngineEvent::TerminationRequested(
                        crate::Termination::ExceededMaxIterations,
                    )
                }
                _ => {}
            }
        }
        EngineEvent::Pass
    }
}
