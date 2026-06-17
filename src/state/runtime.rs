use super::Termination;
use web_time::Duration;

#[derive(Clone)]
pub struct RuntimeState {
    iter: usize,
    max_iter: usize,
    time: Option<Duration>,
    termination: Option<Termination>,
}

impl RuntimeState {
    pub(crate) fn new() -> Self {
        Self {
            iter: 0,
            max_iter: usize::MAX,
            time: None,
            termination: None,
        }
    }

    pub fn iteration(&self) -> usize {
        self.iter
    }

    pub fn increment_iteration(&mut self) {
        self.iter += 1;
    }

    pub fn max_iterations(&self) -> usize {
        self.max_iter
    }

    pub fn set_max_iterations(&mut self, max_iter: usize) {
        self.max_iter = max_iter;
    }

    pub fn duration(&self) -> Option<&Duration> {
        self.time.as_ref()
    }

    pub fn record_duration(&mut self, duration: Duration) {
        self.time = Some(duration);
    }

    pub fn termination(&self) -> Option<Termination> {
        self.termination
    }

    pub fn terminate(&mut self, termination: Termination) {
        self.termination = Some(termination);
    }

    pub fn is_terminated(&self) -> bool {
        self.termination.is_some()
    }

    pub fn exceeded_max_iterations(&self) -> bool {
        self.iter > self.max_iter
    }
}
