use trellis_runner::{
    CancellationGuard, GenerateBuilder, MaxIterationPolicy, Problem, Procedure, Progress,
    StagnationPolicy, UserState,
};

#[derive(Default, Clone, Debug)]
pub struct IntegrationState {
    n: usize,
    estimate: f64,
}

#[derive(thiserror::Error, Debug)]
pub struct IntegrationError;

impl std::fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "integration error")
    }
}

impl UserState for IntegrationState {
    type Float = f64;

    fn progress(&self) -> Progress<Self::Float> {
        Progress::Measure(self.estimate)
    }
}

/// Integrate f(x) = x^2 over [0, 1] using trapezoidal refinement
pub struct TrapezoidalIntegration;

impl Procedure for TrapezoidalIntegration {
    type State = IntegrationState;
    type Problem = ();

    type Output = f64;
    type Error = IntegrationError;

    const NAME: &'static str = "Trapezoidal Integrator";

    fn step(
        &mut self,
        _: &mut Problem<Self::Problem>,
        state: &mut Self::State,
        _guard: CancellationGuard<'_>,
    ) -> Result<(), Self::Error> {
        state.n += 1;

        let n = state.n;
        let h = 1.0 / n as f64;

        let mut sum = 0.0;
        for i in 0..n {
            let x0 = i as f64 * h;
            let x1 = (i + 1) as f64 * h;

            let f0 = x0 * x0;
            let f1 = x1 * x1;

            sum += 0.5 * (f0 + f1) * h;
        }

        state.estimate = sum;

        Ok(())
    }

    fn is_finished(&self, state: &Self::State) -> bool {
        false
    }

    fn finalise(
        &mut self,
        _: &mut Problem<Self::Problem>,
        state: &Self::State,
    ) -> Result<Self::Output, IntegrationError> {
        Ok(state.estimate)
    }
}

fn main() {
    let result = TrapezoidalIntegration
        .build_for(())
        .with_initial_state(IntegrationState::default())
        .and_policy(MaxIterationPolicy::new(3000))
        .and_policy(StagnationPolicy::new(10))
        .finalise()
        .run();

    println!("{result:?}");
}
