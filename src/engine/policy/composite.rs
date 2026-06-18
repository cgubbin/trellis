use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::ProgressReport,
    state::{State, UserState},
    TrellisFloat,
};

pub struct CompositePolicy<S, A, B>
where
    S: UserState,
    A: EnginePolicy<S>,
    B: EnginePolicy<S>,
{
    pub a: A,
    pub b: B,
    _s: std::marker::PhantomData<S>,
}

pub trait PolicyExt<S: UserState>: EnginePolicy<S> + Sized {
    fn and<P>(self, other: P) -> CompositePolicy<S, Self, P>
    where
        P: EnginePolicy<S>;
}

impl<Q, S> PolicyExt<S> for Q
where
    S: UserState,
    Q: EnginePolicy<S> + Sized,
{
    fn and<P>(self, other: P) -> CompositePolicy<S, Self, P>
    where
        P: EnginePolicy<S>,
    {
        CompositePolicy {
            a: self,
            b: other,
            _s: std::marker::PhantomData,
        }
    }
}

impl<S, A, B> EnginePolicy<S> for CompositePolicy<S, A, B>
where
    S: UserState,
    S::Float: TrellisFloat,
    A: EnginePolicy<S>,
    B: EnginePolicy<S>,
{
    fn next(
        &mut self,
        state: &State<S>,
        events: &[RawEvent<S::Float>],
        cancelled: bool,
    ) -> EngineEvent<S::Float> {
        //     match self.a.next(state, progress.clone(), cancelled) {
        //         PolicyDecision::Stop(t) => return PolicyDecision::Stop(t),
        //         PolicyDecision::Pass => self.b.next(state, progress, cancelled),
        //         PolicyDecision::SaveCheckpoint => unimplemented!(),
        //     }
        todo!()
    }
}
