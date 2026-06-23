use trellis_runner::{
    CancellationGuard, GenerateBuilder, MaxIterationPolicy, Procedure, Progress, StagnationPolicy,
    UserState,
};

#[derive(Clone)]
pub struct QuadratureProblem {
    pub a: f64,
    pub b: f64,
    pub f: fn(f64) -> f64,
}

#[derive(Default, Clone, Debug)]
pub struct TrapezoidalState {
    n: usize,
    estimate: f64,
}

impl UserState for TrapezoidalState {
    type Float = f64;

    fn progress(&self) -> Progress<Self::Float> {
        Progress::Measure(self.estimate)
    }
}

/// Integrate f(x) = x^2 over [0, 1] using trapezoidal refinement
pub struct TrapezoidalIntegration;

impl Procedure<QuadratureProblem> for TrapezoidalIntegration {
    type State = TrapezoidalState;

    type Output = f64;

    const NAME: &'static str = "Trapezoidal Integrator";

    fn step(
        &self,
        problem: &mut QuadratureProblem,
        state: &mut Self::State,
        _guard: CancellationGuard<'_>,
    ) {
        state.n += 1;

        let h = (problem.b - problem.a) / state.n as f64;

        let mut sum = 0.0;

        for i in 0..state.n {
            let x0 = problem.a + i as f64 * h;
            let x1 = problem.a + (i + 1) as f64 * h;

            let f0 = (problem.f)(x0);
            let f1 = (problem.f)(x1);

            sum += 0.5 * (f0 + f1) * h;
        }

        state.estimate = sum;
    }

    fn finalise(&self, _: &mut QuadratureProblem, state: &Self::State) -> Self::Output {
        state.estimate
    }
}

fn main() {
    let problem = QuadratureProblem {
        a: 0.0,
        b: 1.0,
        f: |x| x * x,
    };
    let result = TrapezoidalIntegration
        .build_for(problem)
        .with_initial_state(TrapezoidalState::default())
        .and_policy(MaxIterationPolicy::new(3000))
        .and_policy(StagnationPolicy::new(10))
        .finalise()
        .run();

    println!("{result:?}");
}
