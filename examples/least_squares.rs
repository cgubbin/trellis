use trellis_runner::{
    CancellationGuard, GenerateBuilder, MaxIterationPolicy, Problem, Procedure, Progress,
    ProgressDiagnostics, StagnationPolicy, UserState,
};

#[derive(Clone, Debug)]
pub struct LSState {
    a: f64,
    b: f64,
    iter: usize,
    loss: f64,
}

impl Default for LSState {
    fn default() -> Self {
        Self {
            a: 0.0,
            b: 0.0,
            iter: 0,
            loss: f64::INFINITY,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub struct LSError;

impl std::fmt::Display for LSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ls error")
    }
}

impl UserState for LSState {
    type Float = f64;

    fn progress(&self) -> Progress<Self::Float> {
        println!("Loss {0}", self.loss);
        Progress::Report {
            measure: self.loss,
            diagnostics: ProgressDiagnostics {
                gradient_norm: Some((self.a.powi(2) + self.b.powi(2)).sqrt()),
                step_size: Some(0.01),
                ..Default::default()
            },
        }
    }
}

/// Simple linear regression via gradient descent
pub struct LeastSquares;

impl Procedure for LeastSquares {
    type State = LSState;
    type Problem = Vec<(f64, f64)>;

    type Output = (f64, f64);
    type Error = LSError;

    const NAME: &'static str = "Least Squares Optimisation";

    fn step(
        &mut self,
        problem: &mut Problem<Self::Problem>,
        state: &mut Self::State,
        _guard: CancellationGuard<'_>,
    ) -> Result<(), Self::Error> {
        state.iter += 1;

        let lr = 0.01;
        let mut da = 0.0;
        let mut db = 0.0;
        let mut loss = 0.0;

        for (x, y) in &problem.inner {
            let pred = state.a * x + state.b;
            let err = pred - y;

            loss += err * err;
            da += err * x;
            db += err;
        }

        state.a -= lr * da;
        state.b -= lr * db;
        state.loss = loss;

        Ok(())
    }

    fn is_finished(&self, state: &Self::State) -> bool {
        false
    }

    fn finalise(
        &mut self,
        _: &mut Problem<Self::Problem>,
        state: &Self::State,
    ) -> Result<Self::Output, Self::Error> {
        Ok((state.a, state.b))
    }
}

fn main() {
    let data = vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)];

    let result = LeastSquares
        .build_for(data)
        .and_policy(MaxIterationPolicy::new(3000))
        .and_policy(StagnationPolicy::new(10))
        .with_initial_state(LSState::default())
        .finalise()
        .run();

    println!("fit: {:?}", result);
}
