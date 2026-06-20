use crate::{
    engine::EngineSignal,
    procedure::Procedure,
    state::{State, StateView, UserState},
};

pub struct Extensions<S>(Vec<Box<dyn EngineSink<S>>>);

impl<S> Extensions<S>
where
    S: UserState,
{
    pub(super) fn new() -> Self {
        Self(vec![])
    }

    pub(super) fn add<E>(mut self, extension: E) -> Self
    where
        E: EngineSink<S> + 'static,
    {
        self.0.push(Box::new(extension));
        self
    }

    pub(super) fn dispatch(
        &mut self,
        state: StateView<'_, S>,
        signal: &EngineSignal<<S as UserState>::Float>,
    ) {
        for each in &mut self.0 {
            each.handle(state, signal);
        }
    }
}

pub trait EngineSink<S>
where
    S: UserState,
{
    fn handle(&mut self, state: StateView<'_, S>, signal: &EngineSignal<<S as UserState>::Float>);
}
