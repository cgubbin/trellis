use crate::{
    engine::{extensions::EngineSink, EngineSignal},
    state::{
        ConvergenceState, RuntimeState, Snapshotable, State, StateRestorer, StateView, UserState,
    },
};

use std::time::SystemTime;

mod in_memory;

#[cfg(feature = "writing")]
mod json;

pub use in_memory::InMemoryCheckpointStore;

#[cfg(feature = "writing")]
pub use json::JsonCheckpointStore;

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("filesystem error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("serde json error: {0}")]
    SerdeJson(Box<dyn std::error::Error + 'static>),
    #[error("attempted to load, but no checkpoints available")]
    NoCheckpoint,
}

pub struct CheckpointExtension<C> {
    store: C,
}

impl<C> CheckpointExtension<C> {
    pub(crate) fn new(store: C) -> Self {
        Self { store }
    }
}

impl<S, C> EngineSink<S> for CheckpointExtension<C>
where
    S: UserState + Snapshotable,
    C: CheckpointBackend<<S as Snapshotable>::Snapshot, <S as UserState>::Float> + 'static,
{
    fn handle(&mut self, state: StateView<'_, S>, signal: &EngineSignal<<S as UserState>::Float>) {
        if let EngineSignal::CheckpointRequested(_) = signal {
            let checkpoint = CheckpointView::from(state);

            if let Err(e) = self.store.save(checkpoint) {
                eprintln!("error saving checkpoint: {e:?}");
            }
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CheckpointView<'a, SN, F> {
    user: SN,
    runtime: &'a RuntimeState,
    convergence: &'a ConvergenceState<F>,
    version: u32,
    timestamp: SystemTime,
}

impl<'a, S> From<StateView<'a, S>> for CheckpointView<'a, S::Snapshot, <S as UserState>::Float>
where
    S: Snapshotable + UserState,
{
    fn from(state: StateView<'a, S>) -> CheckpointView<'a, S::Snapshot, <S as UserState>::Float> {
        Self {
            user: state.user().snapshot(),
            runtime: state.runtime(),
            convergence: state.convergence(),
            version: 1,
            timestamp: SystemTime::now(),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Checkpoint<SN, F> {
    pub(super) user: SN,
    pub(super) runtime: RuntimeState,
    pub(super) convergence: ConvergenceState<F>,
    version: u32,
    timestamp: SystemTime,
}

impl<SN, F> Checkpoint<SN, F> {
    pub fn new(view: CheckpointView<'_, SN, F>) -> Self
    where
        F: Clone,
    {
        Self {
            user: view.user,
            runtime: view.runtime.clone(),
            convergence: view.convergence.clone(),
            version: view.version,
            timestamp: view.timestamp,
        }
    }
}

impl<SN, F> Checkpoint<SN, F> {
    pub fn into_state<S>(self) -> State<S>
    where
        S: StateRestorer<S> + Snapshotable<Snapshot = SN> + UserState<Float = F>,
    {
        State {
            user: S::restore(self.user),
            runtime: self.runtime,
            convergence: self.convergence,
        }
    }
}

pub trait CheckpointBackend<SN, F>: Send + Sync {
    fn save(&self, checkpoint: CheckpointView<'_, SN, F>) -> Result<(), CheckpointError>;

    fn load(&self) -> Result<Option<Checkpoint<SN, F>>, CheckpointError>;
}

pub trait EngineInitializer<S>
where
    S: UserState,
{
    fn try_load(&self) -> Result<Option<State<S>>, CheckpointError>;
}

impl<S, C> EngineInitializer<S> for C
where
    S: UserState + Snapshotable + StateRestorer<S>,
    C: CheckpointBackend<<S as Snapshotable>::Snapshot, <S as UserState>::Float>,
{
    fn try_load(&self) -> Result<Option<State<S>>, CheckpointError> {
        let maybe_checkpoint = self.load()?;

        Ok(maybe_checkpoint.map(|checkpoint| checkpoint.into_state()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        CancellationGuard, CheckpointPolicy, GenerateBuilder, MaxIterationPolicy, Procedure,
        Progress, ProgressDiagnostics, Snapshotable, StateRestorer, UserState,
    };

    #[derive(Clone, Debug)]
    pub struct DummyProblem {
        pub target: f64,
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct DummySnapshot {
        pub value: f64,
        pub steps: usize,
    }

    #[derive(Clone, Debug)]
    pub struct DummyState {
        pub value: f64,
        pub steps: usize,
    }

    impl Default for DummyState {
        fn default() -> Self {
            Self {
                value: 512.0,
                steps: 0,
            }
        }
    }

    impl UserState for DummyState {
        type Float = f64;

        fn progress(&self) -> Progress<Self::Float> {
            Progress::Report {
                measure: self.value,
                diagnostics: ProgressDiagnostics {
                    absolute_error: Some(self.value.abs()),
                    relative_error: Some(self.value.abs() / 10.0),
                    ..Default::default()
                },
            }
        }
    }

    impl Snapshotable for DummyState {
        type Snapshot = DummySnapshot;

        fn snapshot(&self) -> Self::Snapshot {
            DummySnapshot {
                value: self.value,
                steps: self.steps,
            }
        }
    }

    impl StateRestorer<DummyState> for DummyState {
        fn restore(snapshot: DummySnapshot) -> Self {
            Self {
                value: snapshot.value,
                steps: snapshot.steps,
            }
        }
    }

    pub struct DummyProcedure;

    impl Procedure<DummyProblem> for DummyProcedure {
        const NAME: &'static str = "Dummy Procedure";

        type State = DummyState;
        type Output = DummyState;

        fn initialise(&self, _problem: &mut DummyProblem, _state: &mut Self::State) {}

        fn step(
            &self,
            problem: &mut DummyProblem,
            state: &mut Self::State,
            _guard: CancellationGuard<'_>,
        ) {
            state.steps += 1;

            let delta = state.value - problem.target;

            if delta.abs() > 1e-12 {
                state.value -= 0.5 * delta;
            }
        }

        fn finalise(&self, _problem: &mut DummyProblem, state: &Self::State) -> Self::Output {
            state.clone()
        }
    }

    #[test]
    fn checkpoint_is_saved_when_requested() {
        let store = InMemoryCheckpointStore::new();
        let target = 1.0;

        let _ = DummyProcedure
            .build_for(DummyProblem { target })
            .with_initial_state(DummyState::default())
            .with_checkpoint_backend(store.clone())
            .and_policy(CheckpointPolicy::every(2))
            .and_policy(MaxIterationPolicy::new(3))
            .finalise()
            .run();

        assert_eq!(store.saved_count(), 1);
    }

    #[test]
    fn restored_snap_initialises_user_state() {
        let store = InMemoryCheckpointStore::new();
        let target = 0.0;

        let _ = DummyProcedure
            .build_for(DummyProblem { target })
            .with_initial_state(DummyState::default())
            .with_checkpoint_backend(store.clone())
            .and_policy(CheckpointPolicy::every(2))
            .and_policy(MaxIterationPolicy::new(3))
            .finalise()
            .run();

        assert_eq!(store.saved_count(), 1);

        let snap = DummySnapshot {
            value: 42.0,
            steps: 10,
        };

        let result = DummyProcedure
            .build_for(DummyProblem { target })
            .resume_from_snapshot(snap)
            .and_policy(MaxIterationPolicy::new(1))
            .with_checkpoint_backend(store.clone())
            .finalise()
            .run()
            .unwrap();

        assert_eq!(result.result.value, 21.0);
    }

    #[test]
    fn restored_checkpoint_initialises_user_state() {
        let store = InMemoryCheckpointStore::new();
        let target = 0.0;

        let _ = DummyProcedure
            .build_for(DummyProblem { target })
            .with_initial_state(DummyState::default())
            .with_checkpoint_backend(store.clone())
            .and_policy(CheckpointPolicy::every(2))
            .and_policy(MaxIterationPolicy::new(3))
            .finalise()
            .run();

        assert_eq!(store.saved_count(), 1);

        let result = DummyProcedure
            .build_for(DummyProblem { target })
            .resume_from_checkpoint(store.clone())
            .unwrap()
            .and_policy(MaxIterationPolicy::new(3))
            .with_checkpoint_backend(store.clone())
            .finalise()
            .run()
            .unwrap();

        assert_eq!(result.result.value, 64.0);
    }
}
