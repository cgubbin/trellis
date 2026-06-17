use num_traits::float::FloatCore;
use tokio_util::sync::CancellationToken;

use super::{DefaultEnginePolicy, Engine, Error};
use crate::{
    watchers::{Frequency, Observable, Observer, ObserverVec},
    Control, Problem, Procedure, State, UserState,
};

pub trait GenerateBuilder: Sized + Procedure
where
    Self::State: UserState,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self>;
}

impl<P> GenerateBuilder for P
where
    P: Procedure,
    P::State: UserState,
    <P::State as UserState>::Float: FloatCore,
{
    fn build_for(self, problem: Self::Problem) -> Builder<Self> {
        Builder {
            problem,
            procedure: self,
            state: State::new(),
            time: true,
            cancellation_token: None,
            observers: ObserverVec::default(),
        }
    }
}

pub struct Builder<P>
where
    P: Procedure,
    P::State: UserState,
{
    procedure: P,
    problem: P::Problem,
    state: State<P::State>,
    time: bool,
    cancellation_token: Option<CancellationToken>,
    observers: ObserverVec<State<P::State>>,
}
impl<P> Builder<P>
where
    P: Procedure,
    P::State: UserState,
{
    #[must_use]
    pub fn time(mut self, time: bool) -> Self {
        self.time = time;
        self
    }

    /// Configure the attached state.
    ///
    /// Apply any runtime configuration option to the attached state.
    #[must_use]
    pub fn configure<F: FnOnce(State<P::State>) -> State<P::State>>(
        mut self,
        configure: F,
    ) -> Self {
        let state = configure(self.state);
        self.state = state;
        self
    }

    #[must_use]
    pub fn attach_observer<OBS: Observer<State<P::State>> + 'static>(
        mut self,
        observer: OBS,
        frequency: Frequency,
    ) -> Self {
        self.observers.attach(
            std::sync::Arc::new(std::sync::Mutex::new(observer)),
            frequency,
        );
        self
    }
}

#[cfg(feature = "ctrlc")]
fn attach_ctrlc(token: CancellationToken) -> Result<(), Error> {
    ctrlc::set_handler(move || {
        token.cancel();
    })?;
    Ok(())
}

impl<P> Builder<P>
where
    P: Procedure,
    P::State: UserState,
{
    #[must_use]
    pub fn with_cancellation_token(self, cancellation_token: CancellationToken) -> Builder<P> {
        Builder {
            procedure: self.procedure,
            problem: self.problem,
            state: self.state,
            time: self.time,
            cancellation_token: Some(cancellation_token),
            observers: self.observers,
        }
    }

    pub fn finalise(self) -> Result<Engine<P, DefaultEnginePolicy>, Error> {
        let cancellation = self.cancellation_token.unwrap_or(CancellationToken::new());

        #[cfg(feature = "ctrlc")]
        {
            let _ = attach_ctrlc(cancellation.clone())?;
        }

        let mut engine = Engine {
            problem: Problem::new(self.problem),
            procedure: self.procedure,
            state: Some(self.state),
            time: self.time,
            start_time: None,
            // policy: DefaultConvergencePolicy::default(),
            policy: todo!(),
            cancellation,
            observers: self.observers,
        };
        Ok(engine)
    }
}
