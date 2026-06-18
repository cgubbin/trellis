use super::EnginePolicy;

use crate::{
    engine::{EngineEvent, RawEvent},
    progress::{Progress, ProgressReport},
    state::{State, UserState},
    Termination,
};

use num_traits::float::FloatCore;

pub struct NoProgressPolicy<F> {
    tolerance: F,
    patience: usize,

    last_value: Option<F>,
    counter: usize,
}

impl<F> NoProgressPolicy<F> {
    pub fn new(tolerance: F, patience: usize) -> Self {
        Self {
            tolerance,
            patience,
            last_value: None,
            counter: 0,
        }
    }
}

impl<S> EnginePolicy<S> for NoProgressPolicy<S::Float>
where
    S: UserState,
    <S as UserState>::Float: FloatCore,
{
    fn next(
        &mut self,
        _state: &State<S>,
        events: &[RawEvent<S::Float>],
        _cancelled: bool,
    ) -> EngineEvent<S::Float> {
        for each in events {
            match each {
                RawEvent::Progress(progress) => {
                    let value = match progress {
                        Progress::Metric { value } => value,
                        Progress::ErrorEstimate { absolute, .. } => absolute,
                        _ => continue,
                    };

                    if let Some(previous) = self.last_value {
                        let improvement = (previous - *value).abs();

                        if improvement < self.tolerance {
                            self.counter += 1;
                        } else {
                            self.counter = 0;
                        }
                    }

                    self.last_value = Some(*value);

                    if self.counter >= self.patience {
                        return EngineEvent::TerminationRequested(Termination::Stagnated);
                    }
                }
                _ => {}
            }
        }

        EngineEvent::Pass
    }
}
